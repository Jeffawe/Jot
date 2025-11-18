use std::fs;
use std::path::PathBuf;
use crate::plugin::base_plugin::{ExternalPlugin, Plugin, CommandContext, DaemonContext, LlmContext};
use crate::types::{SearchResult, PluginAction};
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").expect("HOME not set");
        let plugin_dir = PathBuf::from(home).join(".jotx").join("plugins");
        
        fs::create_dir_all(&plugin_dir).ok();
        
        let mut manager = Self {
            plugins: Vec::new(),
            plugin_dir,
        };
        
        // Load all plugins from directory
        manager.load_plugins();
        
        manager
    }
    
    /// Load all plugins from the plugins directory
    fn load_plugins(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Skip non-executable files
                if !path.is_file() {
                    continue;
                }
                
                // Load external plugin
                let name = path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                
                let plugin = ExternalPlugin::new(name, path);
                self.plugins.push(Box::new(plugin));
            }
        }
    }
    
    /// Register a Rust-native plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    /// Trigger hook on all plugins
    pub fn trigger_command_captured(&self, context: &CommandContext) -> bool {
        for plugin in &self.plugins {
            match plugin.on_command_captured(context) {
                Ok(PluginAction::Stop) => return false,
                Ok(PluginAction::Skip) => return false,
                Err(e) => eprintln!("Plugin {} error: {}", plugin.name(), e),
                _ => {}
            }
        }
        true
    }
    
    /// Trigger the on_search_after hook on all plugins.
    ///
    /// This will call on_search_after on all plugins, and if any plugin returns an error, it will be printed to stderr.
    ///
    /// # Arguments
    ///
    /// * query - The search query that triggered the hook
    /// * results - The search results that were returned
    /// * context - The context of the search
    ///
    /// # Examples
    ///
    /// 
    pub fn trigger_search_after(&self, query: &str, results: &mut Vec<SearchResult>) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_search_after(query, results) {
                eprintln!("Plugin {} error: {}", plugin.name(), e);
            }
        }
    }
    
    pub fn trigger_llm_before(&self, prompt: &str, context: &LlmContext) -> bool {
        for plugin in &self.plugins {
            match plugin.on_llm_before(prompt, context) {
                Ok(PluginAction::Stop) => return false,
                Ok(PluginAction::Skip) => return false,
                Err(e) => eprintln!("Plugin {} error: {}", plugin.name(), e),
                _ => {}
            }
        }
        true
    }
    
    pub fn trigger_daemon_tick(&self, context: &DaemonContext) {
        for plugin in &self.plugins {
            if let Err(e) = plugin.on_daemon_tick(context) {
                eprintln!("Plugin {} error: {}", plugin.name(), e);
            }
        }
    }
    
    /// List all loaded plugins
    pub fn list(&self) -> Vec<String> {
        self.plugins.iter().map(|p| p.name().to_string()).collect()
    }
}

// Global plugin manager singleton
pub static GLOBAL_PLUGIN_MANAGER: Lazy<Mutex<PluginManager>> = Lazy::new(|| {
    Mutex::new(PluginManager::new())
});