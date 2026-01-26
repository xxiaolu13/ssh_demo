use std::env;
use futures::future::join_all;
use chrono::{DateTime, Duration, Utc};
use cron_parser::parse;
use log::{info,debug};
use sqlx::PgPool;
use crate::domain::scheduler::JobScheduler;
use crate::repository::ssh::batch_server_ssh_back;
use crate::{domain::ssh_configuration::Message, repository::ssh::{single_server_ssh_back}};
use crate::repository::server::*;
use crate::utils::crypto::*;
use crate::domain::server::ServiceTerminal;
use bytes::Bytes;
use crate::domain::cron_job::CronJob;
use dotenvy::dotenv;


pub fn judge_time(time:DateTime<Utc>) -> bool{
    dotenv().ok();
    let sec = env::var("SAVE_SECS").unwrap_or("3600".to_string()).parse().expect("RELOAD_SECS must be number");
    time - Utc::now() <= Duration::seconds(sec)
    // 下次执行时间 - 当前时间 < reload sql 间隔
}


// 获取时间差，判断这个表达式多久执行一次,从而区别这个任务需要轮询数据库还是在任务执行后会重新加回redis
// 考虑个极端情况 比如job1 还有1ms就要触发了，但是就在此时 reload了下sql ==> 不希望这次reload会影响可以通过自回归到redis的任务
// false加回redis
// true不加回到redis，所以函数返回true 需要轮询数据库
// pub fn get_timing(job_expression: &str) -> Result<bool,anyhow::Error> {
//     let now = Utc::now();
//
//     // 计算接下来两次执行时间
//     let next_time_1 = parse(&job_expression, &now)?;
//     let next_time_2 = parse(&job_expression, &next_time_1)?;
//
//     // 计算执行周期(两次执行之间的时间差)
//     let period = next_time_2 - next_time_1;
//
//     dotenv().ok();
//     let sec: i64 = env::var("RELOAD_SECS")
//         .unwrap_or("3600".to_string())
//         .parse()
//         .expect("RELOAD_SECS must be number");
//     let reload_duration = Duration::seconds(sec);
//
//     // 如果周期小于 reload 间隔,返回 true(需要在redis中调度),反之在数据库调度
//     // 现在 >符号，true是需要在数据库调度，false则不用
//     Ok(period > reload_duration)
// }

// 这里面不用管 enable，任务执行后的善后处理，如果enable关闭 任务不会执行，除非在执行后的同时关闭了enable出现了竞态，概率较小
// 用于初始化，计算了每个任务的下次时间 并且进行更新
pub async fn reload_single_job(pool: &PgPool,job_id: i32,heap: JobScheduler) -> Result<(), anyhow::Error>{
    let job_expression = sqlx::query!("SELECT cron_expression FROM cronjobs where id = $1",job_id).fetch_one(pool).await?;
    let job_expression = job_expression.cron_expression;
    let next_time = parse(&job_expression, &Utc::now())?;
    let _ = sqlx::query!("UPDATE cronjobs SET next_execute_at = $1 WHERE id = $2",next_time,job_id).execute(pool).await?;
    // 任务执行成功后，自动更新自己的下次执行时间
    info!("Reloaded job {} next execute time",job_id);
    if judge_time(next_time) {// 下次执行时间 - 当前时间 < redis 的保存时间
        heap.add_job(job_id,next_time.timestamp_millis()).await?; // 这块不用修正，符合新逻辑
    }
    Ok(())
}



// pub async fn init_job_from_sql(pool: &PgPool,heap: JobScheduler) -> Result<(), anyhow::Error>{
//     let cronjob_id_expression_list = sqlx::query!("SELECT id,cron_expression,enabled FROM cronjobs").fetch_all(pool).await?;
//     // 过滤enabled
//     let job_list: Vec<(i32,String)> = cronjob_id_expression_list.into_iter().filter(|row|row.enabled).map(|row| (row.id,row.cron_expression)).collect();
//     for job in job_list{
//         let (job_id, _job_expression) = job;
//         // let next_time = parse(&job_expression, &Utc::now())?;
//         // let next_time = next_time - Utc::now();
//         //     dotenv().ok();
//         // let sec: i64 = env::var("RELOAD_SECS")
//         //     .unwrap_or("3600".to_string())
//         //     .parse()
//         //     .expect("RELOAD_SECS must be number");
//         // let reload_duration = Duration::seconds(sec);
//         // if next_time < reload_duration{
//         // // 下次执行时间 - 当前时间 > 轮询时间   ==> 需要通过reload sql管理的任务（比如任务 每3h执行1次，轮询时间1h，那么前两次轮询可能不会轮询到这个任务的轮次，当第三次时，reload_single_job内部会让这个任务进入redis）
//         //     debug!("job {} reloaded from sql", job_id);
//         //     let _  = reload_single_job(pool,job_id,heap.clone()).await?;
//         // }
//         debug!("job {} reloaded from sql", job_id);
//         let _  = reload_single_job(pool,job_id,heap.clone()).await?;
//         let tasks = job_list.iter().map(|job| reload_single_job());
//
//     }
//     join_all(tasks).await;
//     Ok(())
// }


// 初始化操作
pub async fn init_job_from_sql(pool: &PgPool, heap: JobScheduler) -> Result<(), anyhow::Error> {
    let cronjob_id_expression_list = sqlx::query!("SELECT id,cron_expression,enabled FROM cronjobs")
        .fetch_all(pool)
        .await?;
    let job_list: Vec<(i32, String)> = cronjob_id_expression_list
        .into_iter()
        .filter(|row| row.enabled)
        .map(|row| (row.id, row.cron_expression))
        .collect();
    let tasks: Vec<_> = job_list
        .into_iter()
        .map(|(job_id, _job_expression)| {
            let pool = pool.clone();  // clone 引用计数
            let heap = heap.clone();
            async move {
                debug!("job {} reloaded from sql", job_id);
                reload_single_job(&pool, job_id, heap).await
            }
        })
        .collect();
    let results = join_all(tasks).await;
    for result in results {
        result?;
    }
    Ok(())
}



// pub async fn reload_job_from_sql(pool: &PgPool,heap: JobScheduler) -> Result<(), anyhow::Error>{
// let cronjob_id_expression_list = sqlx::query!("SELECT id,cron_expression,enabled FROM cronjobs").fetch_all(pool).await?;
//     let job_list: Vec<(i32,String)> = cronjob_id_expression_list.into_iter().filter(|row|row.enabled).map(|row| (row.id,row.cron_expression)).collect();
//     for job in job_list{
//         let (job_id, job_expression) = job;
//         if get_timing(&job_expression)?{
//             info!("job {} reloaded from sql", job_id);
//             let _  = reload_single_job(pool,job_id,heap.clone()).await?;
//         }
//     }
//     Ok(())
// }

// 定时 reload。redis存近3min的任务，每1min循环一次数据库
pub async fn reload_job_from_sql(pool: &PgPool,heap: JobScheduler,save_secs: u64) -> Result<(), anyhow::Error>{
    let save_time = Utc::now() + Duration::seconds(save_secs as i64);
    let due_job = sqlx::query!(
        r#"
    SELECT id,next_execute_at  FROM cronjobs
    WHERE enabled = true
    AND next_execute_at <= $1
    "#,
        save_time
    ).fetch_all(pool).await?;
    let job_list = due_job.into_iter().map(|due_job| (due_job.id,due_job.next_execute_at )).collect::<Vec<(i32, DateTime<Utc>)>>();

    let tasks: Vec<_> = job_list
        .into_iter()
        .map(|(job_id,next_execute_at)| {

            let heap = heap.clone();
            async move {
                info!("job {} add to queue from reload sql", job_id);
                heap.add_job(job_id,next_execute_at.timestamp_millis()).await
            }
        })
        .collect();
    let results = join_all(tasks).await;
    for result in results {
        result?;
    }
    Ok(())
}






pub async fn batch_job_execute(
    job_id: Option<i32>,
    pool: &PgPool,
    msg: CronJob
) -> Result<tokio::sync::mpsc::Receiver<Result<Bytes, std::io::Error>>, anyhow::Error> {
    if !msg.enabled{
        return Err(anyhow::anyhow!("Batch job is not enabled"))
    }
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
    let rx = batch_server_ssh_back(job_id, pool,msg, command).await.map_err(|e| anyhow::anyhow!("Failed to get rx: {}", e))?;
    
    Ok(rx)
}



pub async fn single_job_execute(job_id:Option<i32>,pool: &PgPool,msg: CronJob) -> Result<(u32,String), anyhow::Error>{
    if !msg.enabled{
        return Err(anyhow::anyhow!("Single job is not enabled"))
    }
    let command = msg.command;
    let server_id = msg.server_id.ok_or_else(|| anyhow::anyhow!("server_id is required"))?;
    let server = get_server_by_id_db(pool, server_id).await
    .map_err(|e| anyhow::anyhow!("Failed to get server by group_id: {}", e))?;
    let msg = Message::new(server.ssh_user, server.password.clone(),server.port.to_string(), Some(server.ip),None);
    let (code,output) = single_server_ssh_back(job_id ,pool,msg, command.clone()).await.map_err(|e| anyhow::anyhow!("Failed to get output: {}", e))?;

    Ok((code,output))
}