use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::types::{PluginAction, SearchResult};

// ============================================================================
// PLUGIN TRAIT - What all plugins must implement
// ============================================================================

/// Base trait that all plugins must implement
#[allow(dead_code)]
pub trait Plugin: Send + Sync {
    /// Plugin metadata
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    /// Hook implementations (optional - plugins only implement what they need)
    
    /// Called when a shell command is captured
    fn on_command_captured(&self, _context: &CommandContext) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }
    
    /// Called before search is executed
    fn on_search_before(&self, _query: &str) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }
    
    /// Called after search results are returned
    fn on_search_after(&self, _query: &str, _results: &mut Vec<SearchResult>) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }
    
    /// Called before LLM is invoked
    fn on_llm_before(&self, _prompt: &str, _context: &LlmContext) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }
    
    /// Called after LLM returns response
    fn on_llm_after(&self, _prompt: &str, _response: &mut String, _context: &LlmContext) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }
    
    /// Called on main daemon loop iteration
    fn on_daemon_tick(&self, _context: &DaemonContext) -> Result<PluginAction, String> {
        Ok(PluginAction::Continue)
    }
}

// ============================================================================
// CONTEXTS - Data passed to hooks
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContext {
    pub command: String,
    pub working_dir: String,
    pub user: String,
    pub host: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmContext {
    pub provider: String,
    pub model: String,
    pub working_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonContext {
    pub iteration: u64,
    pub uptime_secs: u64,
}

// ============================================================================
// EXTERNAL PLUGIN - Runs external scripts/binaries
// ============================================================================

/// External plugin that executes a script/binary
pub struct ExternalPlugin {
    name: String,
    path: PathBuf,
    hooks: Vec<String>,  // Which hooks this plugin wants to listen to
}

impl ExternalPlugin {
    pub fn new(name: String, path: PathBuf) -> Self {
        // Read plugin manifest to see which hooks it subscribes to
        let hooks = Self::read_hooks(&path);
        
        Self { name, path, hooks }
    }
    
    fn read_hooks(path: &PathBuf) -> Vec<String> {
        // Read plugin.toml next to the plugin binary
        let manifest_path = path.parent().unwrap().join("plugin.toml");
        
        if let Ok(content) = fs::read_to_string(manifest_path) {
            if let Ok(manifest) = toml::from_str::<PluginManifest>(&content) {
                return manifest.hooks;
            }
        }
        
        vec![]  // Default: no hooks
    }
    
    fn execute(&self, hook: &str, input: serde_json::Value) -> Result<PluginResponse, String> {
        let mut  output = Command::new(&self.path)
            .arg(hook)  // Pass hook name as first argument
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn plugin: {}", e))?;
        
        // Write JSON input to stdin
        use std::io::Write;
        if let Some(stdin) = output.stdin.as_mut() {
            stdin.write_all(input.to_string().as_bytes()).ok();
        }
        
        let output = output.wait_with_output()
            .map_err(|e| format!("Failed to wait for plugin: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("Plugin failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        // Parse JSON response
        let response: PluginResponse = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse plugin response: {}", e))?;
        
        Ok(response)
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PluginManifest {
    name: String,
    version: String,
    hooks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginResponse {
    action: String,  // "continue", "stop", "modify", "skip"
    data: Option<serde_json::Value>,
}

// Implement Plugin trait for ExternalPlugin
impl Plugin for ExternalPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "External plugin"
    }
    
    fn on_command_captured(&self, context: &CommandContext) -> Result<PluginAction, String> {
        if !self.hooks.contains(&"on_command_captured".to_string()) {
            return Ok(PluginAction::Continue);
        }
        
        let input = serde_json::to_value(context).unwrap();
        let response = self.execute("on_command_captured", input)?;
        
        match response.action.as_str() {
            "stop" => Ok(PluginAction::Stop),
            "skip" => Ok(PluginAction::Skip),
            _ => Ok(PluginAction::Continue),
        }
    }
    
    fn on_search_after(&self, _query: &str, results: &mut Vec<SearchResult>) -> Result<PluginAction, String> {
        if !self.hooks.contains(&"on_search_after".to_string()) {
            return Ok(PluginAction::Continue);
        }
        
        let input = serde_json::json!({
            "query": _query,
            "results": results,
        });
        
        let response = self.execute("on_search_after", input)?;
        
        // If plugin modified results, update them
        if let Some(data) = response.data {
            if let Ok(new_results) = serde_json::from_value::<Vec<SearchResult>>(data) {
                *results = new_results;
                return Ok(PluginAction::ModifyData);
            }
        }
        
        Ok(PluginAction::Continue)
    }
}

