use chrono::Utc;
use cron_parser::parse;
use log::debug;
use sqlx::PgPool;
use crate::domain::cron_job::{CreateCronJob, CronJob, CronJobExecutor, UpdateCronJob};
use crate::repository::server::get_server_by_id_db;
use crate::repository::servergroup::get_group_by_id_db;
use crate::domain::scheduler::JobScheduler;
use crate::scheduler::prepare::{judge_time, reload_single_job};
use tracing::info;


pub async fn get_all_cronjobs_db(pool:&PgPool) -> Result<Vec<CronJob>, anyhow::Error>{
    let rows = sqlx::query_as!(CronJob,"select * from cronjobs").fetch_all(pool).await?;
    match rows.len(){
        0 => Err(anyhow::Error::msg("get all servers not found")),
        _ => Ok(rows)
    }
}


pub async fn get_cronjob_by_id_db( pool:&PgPool, id: i32) -> Result<CronJob, anyhow::Error>{
    let row = sqlx::query_as!(CronJob,"select * from cronjobs where id=$1", id).fetch_one(pool).await?;
    Ok(row)
}


pub async fn create_cronjob_db(pool: &PgPool, params: CreateCronJob) -> Result<CreateCronJob, anyhow::Error> {
    let next_time = params.next_tick()?;
    match (params.server_id, params.group_id) {
        (Some(sid), Some(gid)) => {
            get_server_by_id_db(pool, sid).await?;
            get_group_by_id_db(pool, gid).await?;
        }
        (Some(sid), None) => {
            get_server_by_id_db(pool, sid).await?;
        }
        (None, Some(gid)) => {
            get_group_by_id_db(pool, gid).await?;
        }
        (None, None) => {
            return Err(anyhow::Error::msg("must provide server_id or group_id"));
        }
    }

    debug!("create new cronjob db");
    let row = sqlx::query!(
        r#"
        INSERT INTO cronjobs (name,cron_expression,server_id,group_id,command,enabled,timeout,retry_count,description,next_execute_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        RETURNING id,name,cron_expression,server_id,group_id,command,enabled,timeout,retry_count,description,next_execute_at
        "#,
        params.name.clone(),
        params.cron_expression.clone(),
        params.server_id,
        params.group_id,
        params.command.clone(),
        params.enabled,
        params.timeout,
        params.retry_count,
        params.description.clone(),
        next_time
    ).fetch_one(pool).await?;
    if row.enabled{ // 要看enabled是否开启
        let heap = JobScheduler::new().await?;
        if judge_time(next_time) {// 下次执行时间 - 当前时间 < redis 的保存时间
            heap.add_job(row.id,next_time.timestamp_millis()).await?;
        }
        info!("created new cronjob: {:?}", row);
    }

    Ok(CreateCronJob{
        name: row.name,
        cron_expression: row.cron_expression,
        server_id: row.server_id,
        group_id: row.group_id,
        command: row.command,
        enabled: row.enabled,
        timeout: row.timeout,
        retry_count: row.retry_count,
        description: row.description,
        next_execute_at: row.next_execute_at,
    })
}



fn check<T>(a: Option<T>, b: Option<T>) -> Option<T> {
    a.or(b)
}
pub async fn update_cronjob_db(pool: &PgPool, id: i32, params: UpdateCronJob) -> Result<CronJob, anyhow::Error> {
    let this_job = get_cronjob_by_id_db(pool, id).await?;
    let name = check(params.name.clone(), this_job.name.clone());
    let cron_expression = if let Some(e) = params.cron_expression {
        e
    }else{
        this_job.cron_expression.clone()
    };
    let group_id = check(params.group_id.clone(), this_job.group_id.clone());
    let server_id = check(params.server_id.clone(), this_job.server_id.clone());
    let command = if let Some(e) = params.command {
        e
    }else {
        this_job.command.clone()
    };
    let enabled = if let Some(e) = params.enabled {
        e
    }else{
        this_job.enabled.clone()
    };
    let timeout = check(params.timeout.clone(), this_job.timeout.clone());
    let retry_count = check(params.retry_count.clone(), this_job.retry_count.clone());
    let description = check(params.description.clone(), this_job.description.clone());
    let next_execute_at = parse(&cron_expression, &Utc::now())?;

    let heap = JobScheduler::new().await?;
    // 如果更改表达式，则重新判断这条任务是否进入heap,并且需要enabled为true
    if cron_expression != this_job.cron_expression && enabled {
        if judge_time(next_execute_at){
            heap.add_job(this_job.id,next_execute_at.timestamp_millis()).await?;
        }else {
            heap.del_job_pending(this_job.id).await?;
            // 意义为 如果开启任务，这个分支代表了下次执行时间大于save time的任务，那么就从redis删除，等待reload进入redis
        }
    }else if enabled != this_job.enabled && enabled == false{ // 修改了enable且为false
        info!("enabled changed..");
        heap.del_job_pending(this_job.id).await?;
    }else if enabled != this_job.enabled && enabled == true{// 修改了enable且为true
        info!("enabled changed..");
        if judge_time(next_execute_at){ // 代表了下次执行时间小于于save time的任务，add进入Redis
            heap.add_job(this_job.id,next_execute_at.timestamp_millis()).await?;
        }
    }
    match (server_id, group_id) {
        (Some(sid), Some(gid)) => {
            get_server_by_id_db(pool, sid).await?;
            get_group_by_id_db(pool, gid).await?;
        }
        (Some(sid), None) => {
            get_server_by_id_db(pool, sid).await?;
        }
        (None, Some(gid)) => {
            get_group_by_id_db(pool, gid).await?;
        }
        (None, None) => {
            return Err(anyhow::Error::msg("must provide server_id or group_id"));
        }
    }
    let row = sqlx::query_as!(
        CronJob,
        "UPDATE cronjobs SET name=$1,cron_expression=$2,group_id=$3,server_id=$4,command=$5,enabled=$6,timeout=$7,retry_count=$8,description=$9,next_execute_at=$10 WHERE id=$11 returning *",
        name,cron_expression,group_id,server_id,command,enabled,timeout,retry_count,description,next_execute_at,id
    ).fetch_one(pool).await?;
    Ok(row)
}




























