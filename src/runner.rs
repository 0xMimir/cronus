use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::task::JoinHandle;

pub struct Cronus {
    is_running: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Cronus {
    pub fn new() -> Self {
        let handle = tokio::spawn(Self::spawn_worker());

        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            handle,
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn start(&self) {
        self.is_running.store(true, Ordering::Release);
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Release);
    }

    pub fn shutdown(self) {
        self.stop();
        self.handle.abort();
    }

    async fn spawn_worker() {
        loop {}
    }
}
