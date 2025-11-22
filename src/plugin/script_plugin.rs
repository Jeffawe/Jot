use crate::types::{PluginAction, SearchResult};
use rhai::{AST, Dynamic, Engine, Scope};
use std::path::PathBuf;
use std::sync::Arc;

use super::base_plugin::{CommandContext, DaemonContext, LlmContext, Plugin};
use super::script_engine::parse_plugin_action;

pub struct ScriptPlugin {
    plugin_name: String,
    engine: Arc<Engine>, // Store a reference to the shared engine
    ast: AST,
}

impl ScriptPlugin {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let engine = crate::plugin::script_engine::SHARED_RHAI_ENGINE.clone();

        // Compile using shared engine
        let script = std::fs::read_to_string(&path)?;
        let ast = engine.compile(&script)?;

        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(Self {
            plugin_name: name,
            engine,
            ast,
        })
    }

    // Helper to call a script function safely
    fn call_script_fn(
        &self,
        fn_name: &str,
        args: impl rhai::FuncArgs, // This handles the tuple args
    ) -> PluginAction {
        // Create a scope (holds variables for the script execution)
        let mut scope = Scope::new();

        // Call the function
        let result: Result<Dynamic, _> = self.engine.call_fn(&mut scope, &self.ast, fn_name, args);

        match result {
            Ok(val) => parse_plugin_action(val),
            Err(_) => {
                // Function likely doesn't exist in the script, which is fine.
                // Or script crashed. We default to Continue.
                PluginAction::Continue
            }
        }
    }
}

impl Plugin for ScriptPlugin {
    fn name(&self) -> &str {
        &self.plugin_name
    }
    fn version(&self) -> &str {
        "0.1.0"
    } // Could read from file metadata
    fn description(&self) -> &str {
        "User script"
    }

    fn on_command_captured(&self, context: &CommandContext) -> Result<PluginAction, String> {
        // Pass context as a clone (it's read-only effectively)
        Ok(self.call_script_fn("on_command_captured", (context.clone() as CommandContext,)))
    }

    fn on_search_after(
        &self,
        query: &str,
        results: &mut Vec<SearchResult>,
    ) -> Result<PluginAction, String> {
        let mut scope = Scope::new();

        // CRITICAL: In Rhai, we pass the vector directly.
        // Because we registered SearchResult type, the script can iterate and modify.
        // We use `call_fn` with the vector.

        let result: Result<Dynamic, _> = self.engine.call_fn(
            &mut scope,
            &self.ast,
            "on_search_after",
            (query.to_string(), results.clone()), // Pass a clone to script
        );

        // Rhai returns the modified array (or whatever the function returns)
        // Syncing mutable references back from Rhai is tricky.
        // The easiest pattern for "Filter/Map" plugins is:
        // Script takes Array -> Returns Modified Array

        if let Ok(modified_val) = result {
            let cast_result = modified_val.try_cast::<Vec<SearchResult>>();

            if let Some(modified_vec) = cast_result {
                *results = modified_vec;
            } else {
                eprintln!("⚠️ Plugin returned invalid type for search results.");
            }
        }

        Ok(PluginAction::Continue)
    }

    fn on_daemon_tick(&self, context: &DaemonContext) -> Result<PluginAction, String> {
        Ok(self.call_script_fn("on_daemon_tick", (context.clone() as DaemonContext,)))
    }

    fn on_llm_before(&self, _prompt: &str, _context: &LlmContext) -> Result<PluginAction, String> {
        Ok(self.call_script_fn("on_llm_before", (_context.clone() as LlmContext, _prompt.to_string(),)))
    }

    fn on_llm_after(&self, _prompt: &str, _response: &mut String, _context: &LlmContext) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }

    fn on_search_before(&self, _query: &str) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }
}
