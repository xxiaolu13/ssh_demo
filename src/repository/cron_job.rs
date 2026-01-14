
use sqlx::PgPool;
use crate::domain::cron_job::{CreateCronJob, CronJob};
use crate::repository::server::get_server_by_id_db;
use crate::repository::servergroup::get_group_by_id_db;


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
    let row = sqlx::query_as!(
        CreateCronJob,
        r#"
        INSERT INTO cronjobs (name,cron_expression,server_id,group_id,command,enabled,timeout,retry_count,description)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
        RETURNING name,cron_expression,server_id,group_id,command,enabled,timeout,retry_count,description
        "#,
        params.name.clone(),
        params.cron_expression.clone(),
        params.server_id,
        params.group_id,
        params.command.clone(),
        params.enabled,
        params.timeout,
        params.retry_count,
        params.description.clone()
    ).fetch_one(pool).await?;

    Ok(row)
}