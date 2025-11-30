pub mod metadata;
pub mod discovery;
pub mod validation;
pub mod guard;
pub mod enabled;

pub use metadata::{PluginEntry, PluginMetadata};
pub use discovery::discover_plugins;
pub use enabled::{disable_plugin, enable_plugin};
pub use guard::spawn_guarded;
