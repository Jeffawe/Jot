use crate::config::GLOBAL_CONFIG;
use crate::plugin::{CommandContext, Plugin};
use crate::types::PluginAction;
use regex::Regex;

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
        let privacy = {
            if let Ok(config) = GLOBAL_CONFIG.read() {
                config.privacy.clone()
            } else {
                return Ok(PluginAction::Continue);
            }
        };

        for pattern in privacy.excludes_contains_string.iter() {
            if context.command.to_lowercase().contains(pattern) {
                return Ok(PluginAction::Skip);
            }
        }

        // Handle exclude_folders - check if working_dir starts with or is within excluded folders
        for pattern in privacy.exclude_folders.iter() {
            let working_dir_lower = context.working_dir.to_lowercase();
            let pattern_lower = pattern.to_lowercase();

            // Exact match
            if working_dir_lower == pattern_lower {
                return Ok(PluginAction::Skip);
            }

            // Check if working_dir is inside the excluded folder
            // e.g., excluded: "/home/user/private" should match "/home/user/private/subfolder"
            let normalized_pattern = pattern_lower.trim_end_matches('/');
            if working_dir_lower.starts_with(&format!("{}/", normalized_pattern)) {
                return Ok(PluginAction::Skip);
            }
        }

        for pattern in privacy.excludes_regex.iter() {
            match Regex::new(pattern) {
                Ok(re) => {
                    if re.is_match(&context.working_dir) {
                        return Ok(PluginAction::Skip);
                    }
                }
                Err(e) => {
                    // Log the error but continue checking other patterns
                    eprintln!("Invalid regex pattern '{}': {}", pattern, e);
                }
            }
        }

        for pattern in privacy.excludes_starts_with_string.iter() {
            if context.command.to_lowercase().starts_with(pattern) {
                return Ok(PluginAction::Skip);
            }
        }

        for pattern in privacy.excludes_ends_with_string.iter() {
            if context.command.to_lowercase().ends_with(pattern) {
                return Ok(PluginAction::Skip);
            }
        }

        Ok(PluginAction::Continue)
    }
}
