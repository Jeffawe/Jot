pub mod base_plugin;
pub mod plugin_manager;
pub mod sensitive_info_plugin;

pub use plugin_manager::GLOBAL_PLUGIN_MANAGER;
pub use base_plugin::{DaemonContext, CommandContext};
pub use sensitive_info_plugin::SensitiveCommandFilter;
pub use base_plugin::Plugin;