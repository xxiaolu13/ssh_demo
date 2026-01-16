use chrono::{DateTime, Duration, Utc};
use cron_parser::parse;
use sqlx::PgPool;
use crate::domain::scheduler::JobScheduler;

pub fn judge_time(time:DateTime<Utc>) -> bool{
    time - Utc::now() <= Duration::hours(1)
}


pub async fn reload_job_from_sql(pool: &PgPool,heap: JobScheduler) -> Result<(), anyhow::Error>{
let cronjob_id_expression_list = sqlx::query!("SELECT id,cron_expression FROM cronjobs").fetch_all(pool).await?;
    let job_list: Vec<(i32,String)> = cronjob_id_expression_list.into_iter().map(|row| (row.id,row.cron_expression)).collect();
    for job in job_list{

        let (id,expression) = job;

        let next_time = parse(&expression, &Utc::now())?;

        let _ = sqlx::query!("UPDATE cronjobs SET next_execute_at = $1 WHERE id = $2",next_time,id).execute(pool).await?;

        if judge_time(next_time) { // 判断小于1h，这个时间可以写活
            // binaryheap.push(CronWorker{next_execute_at: next_time, cronjob_id: id})?;
            heap.add_job(id,next_time.timestamp_millis()).await?
        }
    }
    Ok(())
}