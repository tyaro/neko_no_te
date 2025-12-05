use std::cmp::Reverse;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use libloading::Library;
use prompt_spi::{
    factory_from_raw, CreatePromptBuilderFactory, PromptAgentMode, PromptBuilder,
    PromptBuilderFactory, PromptMetadata,
};

use super::metadata::{PluginEntry, PluginKind, PluginMetadata};

const DEFAULT_ENTRYPOINT: &str = "create_prompt_builder";

pub struct PromptBuilderRegistry {
    by_model: HashMap<String, Vec<PromptBuilderSource>>,
}

#[derive(Clone)]
pub enum PromptBuilderSource {
    Plugin(Arc<PromptBuilderHandle>),
    Host(Arc<HostPromptBuilderFactory>),
}

impl PromptBuilderRegistry {
    pub fn from_plugins(entries: &[PluginEntry]) -> Self {
        let mut by_model: HashMap<String, Vec<PromptBuilderSource>> = HashMap::new();

        for entry in entries {
            if !entry.enabled {
                continue;
            }
            let manifest = match &entry.metadata {
                Some(m) => m.clone(),
                None => continue,
            };
            if manifest.kind != PluginKind::PromptBuilder {
                continue;
            }

            match PromptBuilderHandle::load(entry, manifest) {
                Ok(handle) => {
                    let handle = Arc::new(handle);
                    for model in &handle.manifest_models {
                        by_model
                            .entry(model.clone())
                            .or_default()
                            .push(PromptBuilderSource::Plugin(handle.clone()));
                    }
                }
                Err(err) => {
                    eprintln!(
                        "failed to load prompt builder plugin '{}': {}",
                        entry.dir_name, err
                    );
                }
            }
        }

        for handles in by_model.values_mut() {
            handles.sort_by_key(|source| Reverse(source.priority()));
        }

        Self { by_model }
    }

    pub fn register_host_builder(
        &mut self,
        model: impl Into<String>,
        factory: HostPromptBuilderFactory,
    ) {
        let entry = PromptBuilderSource::Host(Arc::new(factory));
        let models = self.by_model.entry(model.into()).or_default();
        models.push(entry);
        models.sort_by_key(|source| Reverse(source.priority()));
    }

    pub fn resolve(&self, model: &str) -> Option<PromptBuilderSource> {
        self.by_model
            .get(model)
            .and_then(|list| list.first().cloned())
    }

    pub fn is_empty(&self) -> bool {
        self.by_model.is_empty()
    }
}

impl PromptBuilderSource {
    pub fn create_builder(&self) -> Box<dyn PromptBuilder> {
        match self {
            PromptBuilderSource::Plugin(handle) => handle.create_builder(),
            PromptBuilderSource::Host(factory) => factory.create_builder(),
        }
    }

    pub fn metadata(&self) -> PromptMetadata {
        match self {
            PromptBuilderSource::Plugin(handle) => handle.metadata().clone(),
            PromptBuilderSource::Host(factory) => factory.metadata().clone(),
        }
    }

    pub fn manifest(&self) -> Option<PluginMetadata> {
        match self {
            PromptBuilderSource::Plugin(handle) => Some(handle.manifest().clone()),
            PromptBuilderSource::Host(_) => None,
        }
    }

    pub fn preferred_agent(&self) -> PromptAgentMode {
        match self {
            PromptBuilderSource::Plugin(handle) => handle.preferred_agent(),
            PromptBuilderSource::Host(factory) => factory.preferred_agent(),
        }
    }

    pub fn plugin_dir(&self) -> Option<PathBuf> {
        match self {
            PromptBuilderSource::Plugin(handle) => Some(handle.plugin_dir.clone()),
            PromptBuilderSource::Host(_) => None,
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            PromptBuilderSource::Plugin(handle) => handle.priority(),
            PromptBuilderSource::Host(factory) => factory.priority,
        }
    }

    pub fn origin_label(&self) -> &'static str {
        match self {
            PromptBuilderSource::Plugin(_) => "plugin",
            PromptBuilderSource::Host(factory) => factory.origin_label,
        }
    }
}

pub struct HostPromptBuilderFactory {
    metadata: PromptMetadata,
    preferred_agent: PromptAgentMode,
    pub priority: i32,
    origin_label: &'static str,
    constructor: Arc<dyn Fn() -> Box<dyn PromptBuilder> + Send + Sync>,
}

impl HostPromptBuilderFactory {
    pub fn new(
        metadata: PromptMetadata,
        preferred_agent: PromptAgentMode,
        priority: i32,
        origin_label: &'static str,
        constructor: impl Fn() -> Box<dyn PromptBuilder> + Send + Sync + 'static,
    ) -> Self {
        Self {
            metadata,
            preferred_agent,
            priority,
            origin_label,
            constructor: Arc::new(constructor),
        }
    }

    pub fn create_builder(&self) -> Box<dyn PromptBuilder> {
        (self.constructor)()
    }

    pub fn metadata(&self) -> &PromptMetadata {
        &self.metadata
    }

    pub fn preferred_agent(&self) -> PromptAgentMode {
        self.preferred_agent
    }
}

pub struct PromptBuilderHandle {
    manifest: PluginMetadata,
    runtime_metadata: PromptMetadata,
    manifest_models: Vec<String>,
    factory: Arc<dyn PromptBuilderFactory>,
    pub plugin_dir: PathBuf,
}

impl PromptBuilderHandle {
    fn load(entry: &PluginEntry, manifest: PluginMetadata) -> Result<Self> {
        let library_name = manifest
            .library
            .clone()
            .ok_or_else(|| anyhow!("plugin missing library field"))?;
        let library_path = entry.path.join(library_name);
        if !library_path.exists() {
            return Err(anyhow!("library not found: {}", library_path.display()));
        }

        let entrypoint = manifest
            .entrypoint
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ENTRYPOINT.to_string());

        unsafe {
            let library = Library::new(&library_path)
                .with_context(|| format!("loading library {}", library_path.display()))?;
            let symbol_name = CString::new(entrypoint.clone())
                .map_err(|_| anyhow!("invalid entrypoint '{}': contains null byte", entrypoint))?;
            let constructor: libloading::Symbol<CreatePromptBuilderFactory> = library
                .get(symbol_name.as_bytes_with_nul())
                .with_context(|| format!("resolving symbol '{}'", entrypoint))?;
            let factory_ptr = constructor();
            if factory_ptr.is_null() {
                return Err(anyhow!("entrypoint '{}' returned null", entrypoint));
            }
            let factory_box = factory_from_raw(factory_ptr);
            let runtime_metadata = factory_box.metadata();
            let factory_arc: Arc<dyn PromptBuilderFactory> = factory_box.into();
            std::mem::forget(library);

            Ok(Self {
                manifest_models: manifest.models.clone(),
                manifest,
                runtime_metadata,
                factory: factory_arc,
                plugin_dir: entry.path.clone(),
            })
        }
    }

    pub fn create_builder(&self) -> Box<dyn PromptBuilder> {
        self.factory.create()
    }

    pub fn metadata(&self) -> &PromptMetadata {
        &self.runtime_metadata
    }

    pub fn manifest(&self) -> &PluginMetadata {
        &self.manifest
    }

    pub fn priority(&self) -> i32 {
        self.manifest.priority.unwrap_or(0)
    }

    pub fn preferred_agent(&self) -> PromptAgentMode {
        self.runtime_metadata.preferred_agent
    }
}
