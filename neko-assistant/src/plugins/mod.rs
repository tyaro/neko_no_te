pub mod discovery;
pub mod enabled;
pub mod guard;
pub mod metadata;
pub mod validation;

pub use discovery::discover_plugins;
pub use enabled::{disable_plugin, enable_plugin};
pub use metadata::PluginEntry;
// `spawn_guarded` currently unused; keep implementation in `guard` but do not re-export yet.
