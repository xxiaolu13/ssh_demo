use chrono::Utc;
use std::time::Duration;
use tokio_cron_scheduler::{Job,JobScheduler, JobSchedulerError};
use tracing::{error, info, warn};
use uuid::Uuid;

pub async fn run_example(sched: &mut JobScheduler) -> Result<Vec<Uuid>, JobSchedulerError>{
    let mut four_s_job_async = Job::new_async_tz("1/4 * * * * *", Utc, |uuid, mut l| {
        Box::pin(async move {
            info!("I run async every 4 seconds id {:?}", uuid);
            println!("ttttttttttttttttttttttttttttttttttt");
            let next_tick = l.next_tick_for_job(uuid).await;
            match next_tick {
                Ok(Some(ts)) => info!("Next time for 4s is {:?}", ts),
                _ => warn!("Could not get next tick for 4s job"),
            }
        })
    })?;
    let four_s_job_async_clone = four_s_job_async.clone();
    let js = sched.clone();
    info!("4s job id {:?}", four_s_job_async.guid());
    four_s_job_async.on_start_notification_add(&sched, Box::new(move |job_id, notification_id, type_of_notification| {
        let four_s_job_async_clone = four_s_job_async_clone.clone();
        let js = js.clone();
        Box::pin(async move {
            info!("4s Job {:?} ran on start notification {:?} ({:?})", job_id, notification_id, type_of_notification);
            info!("This should only run once since we're going to remove this notification immediately.");
            info!("Removed? {:?}", four_s_job_async_clone.on_start_notification_remove(&js, &notification_id).await);
        })
    })).await?;

    four_s_job_async
        .on_done_notification_add(
            &sched,
            Box::new(|job_id, notification_id, type_of_notification| {
                Box::pin(async move {
                    info!(
                        "4s Job {:?} completed and ran notification {:?} ({:?})",
                        job_id, notification_id, type_of_notification
                    );
                })
            }),
        )
        .await?;

    let four_s_job_guid = four_s_job_async.guid();
    sched.add(four_s_job_async).await?;
    let start = sched.start().await;
    if let Err(e) = start {
        error!("Error starting scheduler {}", e);
        return Err(e);
    }

    let ret = vec![
        four_s_job_guid,
    ];
    Ok(ret)
}

pub async fn stop_example(
    sched: &mut JobScheduler,
    jobs: Vec<Uuid>,
) -> Result<(), JobSchedulerError> {
    tokio::time::sleep(Duration::from_secs(20)).await;

    for i in jobs {
        sched.remove(&i).await?;
    }

    tokio::time::sleep(Duration::from_secs(40)).await;

    info!("Goodbye.");
    sched.shutdown().await?;
    Ok(())
}
