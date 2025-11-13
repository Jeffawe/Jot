use once_cell::sync::Lazy;          // <-- add to Cargo.toml
use std::sync::{Arc, Mutex};

type ShutdownCallback = Box<dyn Fn() + Send + Sync + 'static>;

pub struct ShutdownManager {
    callbacks: Arc<Mutex<Vec<ShutdownCallback>>>,
}

impl ShutdownManager {
    pub fn new() -> Self {
        Self {
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn on_shutdown<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.callbacks.lock().unwrap().push(Box::new(callback));
    }

    pub fn shutdown(&self) {
        println!("Running cleanup handlers...");
        let callbacks = self.callbacks.lock().unwrap();
        for (i, cb) in callbacks.iter().enumerate() {
            println!("  Running cleanup handler {}...", i + 1);
            cb();
        }
        println!("Cleanup complete!");
    }

    // pub fn clone_manager(&self) -> Self {
    //     Self {
    //         callbacks: Arc::clone(&self.callbacks),
    //     }
    // }
}

pub static GLOBAL_SHUTDOWN: Lazy<ShutdownManager> = Lazy::new(|| ShutdownManager::new());

pub fn on_shutdown<F>(callback: F)
where
    F: Fn() + Send + Sync + 'static,
{
    GLOBAL_SHUTDOWN.on_shutdown(callback);
}

pub fn shutdown() {
    GLOBAL_SHUTDOWN.shutdown();
}