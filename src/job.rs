use async_trait::async_trait;
use cron::Schedule;

#[async_trait]
pub trait Job: Send + Sync + 'static {
    fn should_run(&self) -> bool {
        true
    }
    fn schedule(&self) -> Schedule;
    async fn job(&self);
}