use std::{
    collections::HashMap,
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
use uuid::Uuid;

use crate::{
    anonymous_job::AnonymousJob,
    error::{Error, Result},
    job::Job,
};

pub struct Cronus {
    is_running: Arc<AtomicBool>,
    handle: JoinHandle<()>,
    job_sender: Sender<JobMessage>,
}

impl Cronus {
    const RUN_DIFF: Duration = Duration::milliseconds(100);
    const SLEEP: StdDuration = StdDuration::from_millis(100);

    pub fn new() -> Self {
        let is_running = Arc::new(AtomicBool::new(false));
        let (tx, rx) = channel();

        let handle = tokio::spawn(Self::spawn_worker(rx, is_running.clone()));

        Self {
            is_running,
            handle,
            job_sender: tx,
        }
    }

    pub fn remove(&self, id: Uuid) -> Result<()> {
        self.job_sender
            .send(JobMessage::Remove(id))
            .map_err(|_| Error::ErrorRemovingJob)
    }

    pub fn add<J>(&self, job: J) -> Result<Uuid>
    where
        J: Job,
    {
        let id = Uuid::new_v4();
        self.job_sender
            .send(JobMessage::Add {
                job: Box::new(job),
                id,
            })
            .map_err(|_| Error::ErrorAddingJob)?;

        Ok(id)
    }

    pub fn add_anonymous<F, O>(&self, schedule: Schedule, job: F) -> Result<Uuid>
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

    async fn spawn_worker(rx: Receiver<JobMessage>, should_run: Arc<AtomicBool>) {
        let mut jobs = HashMap::new();

        loop {
            if let Ok(job) = rx.recv_timeout(StdDuration::from_millis(10)) {
                // jobs.push(Arc::new(job));
                match job {
                    JobMessage::Add { job, id } => {
                        jobs.insert(id, Arc::new(job));
                    }
                    JobMessage::Remove(id) => {
                        jobs.remove(&id);
                    }
                }
            }

            if should_run.load(Ordering::Relaxed) {
                for (_id, job) in jobs.iter() {
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

enum JobMessage {
    Add { job: Box<dyn Job>, id: Uuid },
    Remove(Uuid),
}
