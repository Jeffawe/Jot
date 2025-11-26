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
        let sensitive_patterns = ["export AWS_SECRET", "password=", "--token", "./target/debug/jotx"];
        
        for pattern in sensitive_patterns {
            println!("Checking command for sensitive pattern: {}", pattern);
            if context.command.contains(pattern) {
                return Ok(PluginAction::Skip);
            }
        }
        
        Ok(PluginAction::Continue)
    }
}