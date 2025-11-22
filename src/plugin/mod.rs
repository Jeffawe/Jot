pub mod base_plugin;
pub mod plugin_manager;
pub mod sensitive_info_plugin;
pub mod script_plugin;
pub mod script_engine;
pub mod create_plugins;
pub mod check_plugins;

pub use plugin_manager::GLOBAL_PLUGIN_MANAGER;
pub use base_plugin::{DaemonContext, CommandContext};
pub use sensitive_info_plugin::SensitiveCommandFilter;
pub use base_plugin::Plugin;
pub use base_plugin::LlmContext;
pub use create_plugins::create_new_plugin_script;
pub use check_plugins::check_plugin_functions;