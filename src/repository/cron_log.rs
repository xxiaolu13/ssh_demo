use sqlx::PgPool;
use crate::domain::cron_log::{CreateCronLog, CronLog};




pub async fn get_cron_log_by_job_id_db(
    pool: &PgPool,
    job_id: i32
) -> Result<Vec<CronLog>, anyhow::Error> {
    let rows = sqlx::query_as!(
        CronLog,
        r#"
        SELECT log_id, job_id,server_ip, status, output, created_at
        FROM cronjob_logs 
        WHERE job_id = $1
        ORDER BY created_at DESC
        "#,
        job_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)  // 即使是空 Vec 也返回 Ok
}

pub async fn get_cron_log_by_server_ip_db(pool: &PgPool,server_ip: String) -> Result<Vec<CronLog>, anyhow::Error>{
    let rows = sqlx::query_as!(
        CronLog,
        "select log_id, job_id,server_ip, status, output, created_at from cronjob_logs where server_ip=$1 ORDER BY created_at DESC",
        server_ip
    ).fetch_all(pool).await?;
    Ok(rows)
}


pub async fn create_cron_log_db(pool: &PgPool,params: CreateCronLog) -> Result<CreateCronLog,anyhow::Error>{
    let row = sqlx::query_as!
    (CreateCronLog,"insert into cronjob_logs (job_id,server_ip,status,output) values ($1,$2,$3,$4) returning job_id,server_ip,status,output"
    ,params.job_id,params.server_ip,params.status,params.output)
    .fetch_one(pool).await?;
    Ok(row)
}