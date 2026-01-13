use actix_web::web;
use sqlx::PgPool;
use sqlx::postgres::PgQueryResult;
use crate::domain::cron_job::{CreateCronJob, CronJob};
use crate::repository::server::get_server_by_id_db;
use crate::repository::servergroup::get_group_by_id_db;


pub async fn get_all_cronjobs_db(pool:&PgPool) -> Result<Vec<CronJob>, anyhow::Error>{
    let rows = sqlx::query_as!(CronJob,"select id,name,cron_expression,server_id,group_id,command from cronjobs").fetch_all(pool).await?;
    match rows.len(){
        0 => Err(anyhow::Error::msg("get all servers not found")),
        _ => Ok(rows)
    }
}


pub async fn get_cronjob_by_id_db( pool:&PgPool, id: i32) -> Result<CronJob, anyhow::Error>{
    let row = sqlx::query_as!(CronJob,"select id,name,cron_expression,server_id,group_id,command from cronjobs where id=$1", id).fetch_one(pool).await?;
    Ok(row)
}




pub async fn create_cronjob_db(pool: &PgPool, params: CreateCronJob) -> Result<CreateCronJob, anyhow::Error> {
    if let Some(sid) = params.server_id && let Some(gid) = params.group_id{
        let _ = get_server_by_id_db(pool,sid).await?;
        let _ = get_group_by_id_db(pool,gid).await?;
    }
    else if let Some(sid) = params.server_id {
        let _ = get_server_by_id_db(pool,sid).await?;
    }
    else if let Some(gid)  = params.group_id {
        let _ = get_group_by_id_db(pool,gid).await?;
    }
    else{
        Err(anyhow::Error::msg("if you create cronjob u must input at least one id,server_id or group_id"))?;
    }
    let row = sqlx::query_as!(
        CreateCronJob,
        r#"
        INSERT INTO cronjobs (name,cron_expression,server_id,group_id,command)
        VALUES ($1, $2,$3,$4,$5)
        RETURNING name,cron_expression,server_id,group_id,command
        "#,
        params.name.clone(),
        params.cron_expression.clone(),
        params.server_id,
        params.group_id,
        params.command.clone()
    ).fetch_one(pool).await?;

    Ok(row)
}