pub mod metadata;
pub mod discovery;
pub mod enabled;

pub use metadata::{PluginEntry, PluginMetadata};
pub use discovery::discover_plugins;
pub use enabled::{disable_plugin, enable_plugin};
