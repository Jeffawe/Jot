use crate::plugin::{CommandContext, Plugin};
use crate::types::PluginAction;

#[allow(dead_code)]
pub struct SensitiveCommandFilter;

impl Plugin for SensitiveCommandFilter {
    fn name(&self) -> &str {
        "sensitive-filter"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn description(&self) -> &str {
        "Blocks capturing of commands with sensitive data"
    }
    
    fn on_command_captured(&self, context: &CommandContext) -> Result<PluginAction, String> {
        let sensitive_patterns = ["export AWS_SECRET", "password=", "--token"];
        
        for pattern in sensitive_patterns {
            if context.command.contains(pattern) {
                println!("🔒 Blocked capturing sensitive command");
                return Ok(PluginAction::Skip);
            }
        }
        
        Ok(PluginAction::Continue)
    }
}