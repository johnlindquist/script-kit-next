pub mod discovery;
pub mod manifest;
pub mod skills;
pub mod types;

pub use discovery::discover_plugins;
#[allow(unused_imports)]
pub use discovery::{
    discover_plugins_in, plugin_agents_dir, plugin_extensions_dir, plugin_scripts_dir,
    plugin_skills_dir, plugins_container_dir,
};
pub use manifest::read_plugin_manifest;
#[allow(unused_imports)]
pub use manifest::synthesize_plugin_manifest;
pub use skills::discover_plugin_skills;
#[allow(unused_imports)]
pub use types::{PluginIndex, PluginManifest, PluginRoot, PluginSkill};
