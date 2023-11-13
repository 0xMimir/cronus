use std::future::Future;

use async_trait::async_trait;
use cron::Schedule;

use crate::Job;

pub struct AnonymousJob<F, O>
where
    F: Fn() -> O,
    O: Future<Output = ()>,
    F: Send + Sync + 'static,
{
    job_function: F,
    schedule: Schedule,
}

impl<F, O> AnonymousJob<F, O>
where
    F: Fn() -> O,
    O: Future<Output = ()>,
    F: Send + Sync + 'static,
{
    pub fn new(schedule: Schedule, job: F) -> Self {
        Self {
            job_function: job,
            schedule,
        }
    }
}

#[async_trait]
impl<F, O> Job for AnonymousJob<F, O>
where
    F: Fn() -> O,
    O: Future<Output = ()>,
    O: 'static + Send + Sync,
    F: Send + Sync + 'static,
{
    fn schedule(&self) -> Schedule {
        self.schedule.clone()
    }
    async fn job(&self) {
        (self.job_function)().await;
    }
}
