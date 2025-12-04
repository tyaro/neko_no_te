pub mod discovery;
pub mod enabled;
pub mod guard;
pub mod metadata;
pub mod prompt_builder;
pub mod validation;

pub use discovery::discover_plugins;
pub use enabled::{disable_plugin, enable_plugin};
pub use metadata::PluginEntry;
pub use prompt_builder::{PromptBuilderRegistry, PromptBuilderSource};
