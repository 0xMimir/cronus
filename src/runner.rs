use std::{sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock,
}, time::Duration};

use tokio::{task::JoinHandle, time::sleep};

use crate::job::Job;

pub struct Cronus {
    jobs: Arc<RwLock<Vec<Box<dyn Job>>>>,
    is_running: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

impl Cronus {
    pub fn new() -> Self {
        let handle = tokio::spawn(Self::spawn_worker());

        Self {
            jobs: Arc::new(RwLock::default()),
            is_running: Arc::new(AtomicBool::new(false)),
            handle,
        }
    }

    pub fn add<T>(&self, job: T) where T: Job{
        // let job = Box::new(job);
        let f = RwLock::new(vec![1]);
        let mut s = f.write().unwrap();
        s.push(3);
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
        loop {



            sleep(Duration::from_millis(1)).await;
        }
    }
}
