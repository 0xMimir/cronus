use cron::Schedule;

pub trait Job {
    fn should_run(&self) -> bool{
        true
    }
    fn schedule(&self) -> Schedule;
    async fn job(&self);
}
