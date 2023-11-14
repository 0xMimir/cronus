use std::{
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    time::Duration,
};

use async_trait::async_trait;
use cron::Schedule;
use cronus::{Cronus, Job};
use tokio::time::sleep;

#[derive(Default)]
pub struct SomeJob {
    counter: Arc<AtomicU8>,
}

#[async_trait]
impl Job for SomeJob {
    fn schedule(&self) -> Schedule {
        "1/2 * * * * *".parse().unwrap()
    }
    async fn job(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

#[tokio::test]
///
/// Create two jobs that run every two seconds and add one to counter
/// Wait for two seconds and see that every job has been ran
/// Remove one job
/// Wait for two seconds and see that only second job has been ran
/// 
async fn job_runs() {
    let cron = Cronus::new();
    let counter = Arc::new(AtomicU8::default());

    let job_id = cron
        .add(SomeJob {
            counter: counter.clone(),
        })
        .expect("Error adding job");

    let counter_arc = counter.clone();
    cron.add_anonymous("1/2 * * * * *".parse().unwrap(), move || {
        let counter = counter_arc.clone();
        async move {
            counter.fetch_add(1, Ordering::Relaxed);
        }
    })
    .expect("Error adding job");

    cron.start();

    sleep(Duration::from_secs(2)).await;

    assert_eq!(counter.load(Ordering::Relaxed), 2);

    cron.remove(job_id).expect("Error removing job");
    sleep(Duration::from_secs(2)).await;
    assert_eq!(counter.load(Ordering::Relaxed), 3);

    cron.shutdown();
}
