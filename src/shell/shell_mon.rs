use once_cell::sync::Lazy;  
use std::sync::Mutex;
use std::collections::VecDeque;

use crate::types::{ShellEntry};

const MAX_HISTORY: usize = 100;


pub struct ShellMon {
    pub history: VecDeque<ShellEntry>,
}

impl ShellMon {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY),
        }
    }

    pub fn check(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Placeholder for shell monitoring logic
        Ok(())
    }
}

pub static GLOBAL_SHELL_MON: Lazy<Mutex<ShellMon>> = Lazy::new(|| {
    Mutex::new(ShellMon::new())
});
