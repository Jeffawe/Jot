use rhai::{Engine, Dynamic};
use crate::types::{SearchResult, PluginAction};
use crate::plugin::base_plugin::{CommandContext, LlmContext, DaemonContext, };
use std::sync::Arc;
use once_cell::sync::Lazy;

pub fn create_engine() -> Engine {
    let mut engine = Engine::new();

    // 1. Register Enums (Helper to convert string returns to Rust Enum)
    // We will let scripts return strings like "stop", "skip", "continue"
    
    // 2. Register CommandContext (Read-Only access is enough usually)
    engine.register_type_with_name::<CommandContext>("CommandContext")
        .register_get("command", |c: &mut CommandContext| c.command.clone())
        .register_get("user", |c: &mut CommandContext| c.user.clone())
        .register_get("working_dir", |c: &mut CommandContext| c.working_dir.clone());

    // 3. Register SearchResult (Needs Getters AND Setters for mutation)
    engine.register_type_with_name::<SearchResult>("SearchResult")
        .register_get("content", |s: &mut SearchResult| s.content.clone())
        .register_set("content", |s: &mut SearchResult, v: String| s.content = v)
        .register_get("similarity", |s: &mut SearchResult| s.similarity)
        .register_set("similarity", |s: &mut SearchResult, v: f32| s.similarity = v);

    engine.register_type_with_name::<LlmContext>("LlmContext")
        .register_get("provider", |c: &mut LlmContext| c.provider.clone())
        .register_get("model", |c: &mut LlmContext| c.model.clone())
        .register_get("working_dir", |c: &mut LlmContext| c.working_dir.clone());

    engine.register_type_with_name::<DaemonContext>("DaemonContext")
        .register_get("iteration", |c: &mut DaemonContext| c.iteration)
        .register_get("uptime_secs", |c: &mut DaemonContext| c.uptime_secs);
        
    // ... Register other types (LlmContext, etc) similarly ...

    engine
}

// Helper to convert script output (String) to PluginAction
pub fn parse_plugin_action(result: Dynamic) -> PluginAction {
    match result.into_string().unwrap_or_default().to_lowercase().as_str() {
        "stop" => PluginAction::Stop,
        "skip" => PluginAction::Skip,
        _ => PluginAction::Continue,
    }
}

// Create one shared engine for the whole app
pub static SHARED_RHAI_ENGINE: Lazy<Arc<Engine>> = Lazy::new(|| {
    Arc::new(create_engine())
});