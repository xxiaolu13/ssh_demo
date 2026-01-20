use chrono::{DateTime, Duration, Utc};
use cron_parser::parse;
use sqlx::PgPool;
use crate::domain::scheduler::JobScheduler;
use crate::repository::ssh::batch_server_ssh_back;
use crate::{domain::ssh_configuration::Message, repository::ssh::{single_server_ssh_back}};
use crate::repository::server::*;
use crate::utils::crypto::*;
use crate::domain::server::ServiceTerminal;
use bytes::Bytes;
use crate::domain::cron_job::CronJob;



pub fn judge_time(time:DateTime<Utc>) -> bool{
    time - Utc::now() <= Duration::hours(1)
}

// pub async fn reload_batch_job(pool: &PgPool,group_id: i32,heap: JobScheduler) -> Result<(), anyhow::Error>{
//     let server_id_list  = get_server_by_group_id_db(pool,group_id).await?;
//     for server in server_id_list{
//         let _ = reload_single_job(pool, server.id, heap.clone());
//     }
//     Ok(())
// }


pub async fn reload_single_job(pool: &PgPool,job_id: i32,heap: JobScheduler) -> Result<(), anyhow::Error>{
    let job_expression = sqlx::query!("SELECT cron_expression FROM cronjobs where id = $1",job_id).fetch_one(pool).await?;
    let job_expression = job_expression.cron_expression;
    let next_time = parse(&job_expression, &Utc::now())?;
    let _ = sqlx::query!("UPDATE cronjobs SET next_execute_at = $1 WHERE id = $2",next_time,job_id).execute(pool).await?;
    if judge_time(next_time) {
            heap.add_job(job_id,next_time.timestamp_millis()).await?;
    }
    Ok(())
}

pub async fn reload_job_from_sql(pool: &PgPool,heap: JobScheduler) -> Result<(), anyhow::Error>{
let cronjob_id_expression_list = sqlx::query!("SELECT id FROM cronjobs").fetch_all(pool).await?;
    let job_list: Vec<i32> = cronjob_id_expression_list.into_iter().map(|row| row.id).collect();
    for job_id in job_list{
        let _  = reload_single_job(pool,job_id,heap.clone()).await?;
    }
    Ok(())
}


pub async fn batch_job_execute(
    pool: &PgPool,
    msg: CronJob
) -> Result<tokio::sync::mpsc::Receiver<Result<Bytes, std::io::Error>>, anyhow::Error> {
    let command = msg.command;
    
    // 如果 group_id 不存在，直接返回错误
    let group_id = msg.group_id.ok_or_else(|| anyhow::anyhow!("group_id is required"))?;
    
    let server_list: Vec<ServiceTerminal> = get_server_by_group_id_db(pool, group_id).await
        .map_err(|e| anyhow::anyhow!("Failed to get server by group_id: {}", e))?;
    
    let ssh_user = server_list[0].ssh_user.clone();
    let password = passwd_decrypt(server_list[0].password.clone())
        .map_err(|e| anyhow::anyhow!("Failed to change password: {}", e))?;
    
    let port = server_list[0].port.to_string();
    let server_list: Vec<String> = server_list.into_iter().map(|e| e.ip.clone()).collect();

    let msg = Message::new(ssh_user, password, port, None, Some(server_list));
    let rx = batch_server_ssh_back(msg, command).await.map_err(|e| anyhow::anyhow!("Failed to get rx: {}", e))?;
    
    Ok(rx)
}



pub async fn single_job_execute(pool: &PgPool,msg: CronJob) -> Result<(u32,String), anyhow::Error>{
    let command = msg.command;
    let server_id = msg.server_id.ok_or_else(|| anyhow::anyhow!("server_id is required"))?;
    let server = get_server_by_id_db(pool, server_id).await
    .map_err(|e| anyhow::anyhow!("Failed to get server by group_id: {}", e))?;
    let msg = Message::new(server.ssh_user, server.password.clone(),server.port.to_string(), Some(server.ip),None);
    let (code,output) = single_server_ssh_back(msg, command.clone()).await.map_err(|e| anyhow::anyhow!("Failed to get output: {}", e))?;
    Ok((code,output))
}