use cron::Schedule;
use async_trait::async_trait;

#[async_trait]
pub trait Job {
    // fn should_run(&self) -> bool{
    //     true
    // }
    fn schedule(&self) -> Schedule;
    // async fn job(&self);
}
