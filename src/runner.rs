use std::{
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    time::Duration as StdDuration,
};

use chrono::{Duration, Utc};
use cron::Schedule;
use tokio::{task::JoinHandle, time::sleep};

use crate::{
    anonymous_job::AnonymousJob,
    error::{Error, Result},
    job::Job,
};

pub struct Cronus {
    is_running: Arc<AtomicBool>,
    handle: JoinHandle<()>,
    job_sender: Sender<Box<dyn Job>>,
}

impl Cronus {
    const RUN_DIFF: Duration = Duration::milliseconds(100);
    const SLEEP: StdDuration = StdDuration::from_millis(100);

    pub fn new() -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        let (tx, rx) = channel::<Box<dyn Job>>();

        let handle = tokio::spawn(Self::spawn_worker(rx, is_running.clone()));

        Self {
            is_running,
            handle,
            job_sender: tx,
        }
    }

    pub fn add<J>(&self, job: J) -> Result<()>
    where
        J: Job,
    {
        self.job_sender
            .send(Box::new(job))
            .map_err(|_| Error::ErrorAddingJob)
    }

    pub fn add_anonymous<F, O>(&self, schedule: Schedule, job: F) -> Result<()>
    where
        F: Fn() -> O,
        O: Future<Output = ()>,
        O: 'static + Send + Sync,
        F: Send + Sync + 'static,
    {
        let job = AnonymousJob::new(schedule, job);
        self.add(job)
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

    async fn spawn_worker(rx: Receiver<Box<dyn Job>>, should_run: Arc<AtomicBool>) {
        let mut jobs = vec![];

        loop {
            if let Ok(job) = rx.recv_timeout(StdDuration::from_millis(10)) {
                jobs.push(Arc::new(job));
            }

            if should_run.load(Ordering::Relaxed) {
                for job in jobs.iter() {
                    let next_run = job
                        .schedule()
                        .upcoming(Utc)
                        .take(1)
                        .next()
                        .expect("This should never fail");

                    let now = Utc::now();
                    let diff = next_run - now;
                    if diff <= Self::RUN_DIFF {
                        let job = job.clone();
                        tokio::spawn(async move {
                            job.job().await;
                        });
                    }
                }
            }

            sleep(Self::SLEEP).await;
        }
    }
}
