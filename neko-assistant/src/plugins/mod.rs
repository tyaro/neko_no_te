pub mod metadata;
pub mod discovery;
pub mod validation;
pub mod guard;
pub mod enabled;

pub use metadata::PluginEntry;
pub use discovery::discover_plugins;
pub use enabled::{disable_plugin, enable_plugin};
// `spawn_guarded` currently unused; keep implementation in `guard` but do not re-export yet.
