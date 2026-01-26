use dotenvy::dotenv;
use log::warn;
use sqlx::PgPool;
use connect_ok::domain::scheduler::JobScheduler;
use tracing::{info, debug, error};
use tokio::signal;
use connect_ok::repository::cron_job::*;
use connect_ok::scheduler::prepare::*;
use anyhow::Result;

// 业务逻辑抽离出来
async fn process_job(pool: &PgPool, heap: &JobScheduler, job_id: i32) -> Result<(),anyhow::Error> {
    // heap.del_job(job_id).await?;
    info!("job {} start execute", job_id);
    // let job_log = CreateCronLog::new(job_id, status, output);
    let msg = get_cronjob_by_id_db(pool, job_id).await?;
    match msg.group_id {
        Some(_) => {
            batch_job_execute(Some(job_id),pool, msg.clone()).await?;
            heap.del_job(msg.id).await?;// 任务完成 从processing移除
        },
        None => {
            single_job_execute(Some(job_id),pool, msg.clone()).await?;
            heap.del_job(msg.id).await?;
            // let (code,output) = single_job_execute(pool, msg.clone()).await?;
            // let job_log = match single_job_execute(pool, msg.clone()).await{
            //     Ok((code,output)) => {
            //         let job_log = CreateCronLog::new(job_id, code.to_string(), Some(output));
            //         heap.del_job(msg.id).await?;// 任务完成 从processing移除
            //         job_log
            //     }
            //     _ => {
            //         let job_log = CreateCronLog::new(job_id, "Error".into(),None);
            //         job_log
            //     }
            // };
            // let _  = create_cron_log_db(pool, job_log).await?;
        }
    }
    reload_single_job(pool, msg.id, heap.clone()).await?;
    Ok(())
}

async fn retry_process_job(pool: &PgPool,heap: &JobScheduler, job_id: i32) -> Result<()>{
    let retry_count = sqlx::query!("select retry_count  from cronjobs where id = $1",job_id)
        .fetch_one(pool).await    
        .map(|row| row.retry_count)  // 提取字段
        .unwrap_or(Some(1));  // 失败时默认值
    if let Some(retry_count) = retry_count{
        info!("job{} start retry",job_id);
        let mut i = 0;
        while i < retry_count {
            i += 1;
            match process_job(pool, heap, job_id).await{
                Ok(_) => {
                    info!("job {} retry {} times success",job_id,i); 
                    return Ok(())
                }
                _ => {
                    error!("job {} retry {} times failed",job_id,i);
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await; // 间隔200ms，这块的时间可能影响大时间的任务
            // 任务在重试机制后，如果失败，就再也不会执行了
        }
        let _ = sqlx::query!("UPDATE cronjobs SET enabled = $1 WHERE id=$2",false,job_id).execute(pool).await;
        error!("job {} all retry failed The job has been actively closed by the program",job_id)
    }
    else {
        warn!("job {} Failed & retry count is None",job_id);
        return Ok(())
    }
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("Process started with PID: {}", std::process::id());
    let db_url = std::env::var("DATABASE_URL").expect("notfound env var DATABASE_URL");
    info!("Using DATABASE_URL: {}", &db_url);
    let pool = PgPool::connect(&db_url).await?;
    let heap = JobScheduler::new().await?;
    heap.clear_all_jobs().await?; // 清空所有队列
    let reload_sec: u64 = std::env::var("RELOAD_SECS")
        .unwrap_or("100".to_string()).parse().expect("RELOAD_SECS must be number");
    let save_sec: u64 = std::env::var("SAVE_SECS")
        .unwrap_or("300".to_string()).parse().expect("SAVE_SECS must be number");
    info!("Worker reloads every {} secs,Redis save {} secs", reload_sec,save_sec);
    // 初始化加载
    let pool1 = pool.clone();
    let heap1 = heap.clone();
    // 首次运行 先reload next execute at,如果不这么做，在执行时候，worker会有任务补偿，将所有任务都执行一遍
    let _ = init_job_from_sql(&pool, heap.clone()).await?;

    // 定时轮询数据库
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(reload_sec));
        interval.tick().await; 
        loop {
            interval.tick().await;
            match reload_job_from_sql(&pool1, heap1.clone(),save_sec).await { // save_secs 是保存时间，小于这个时间需要add heap
                Ok(_) => info!("Reload job from sql success"),
                Err(_) => error!("Failed to reload job from sql!!"),
            };
        }
    });
    // worker启动
    let worker_pool = pool.clone();
    let worker_heap = heap.clone();
    tokio::spawn(async move {
        loop {
            let worker_pool2 = worker_pool.clone();
            let worker_heap2 =worker_heap.clone();
            
            match worker_heap.get_job().await {
                Ok(Some(job_id)) => {
                    info!("job {} shouled run", job_id);
                    tokio::spawn(async move{
                        if let Err(e) = process_job(&worker_pool2, &worker_heap2, job_id).await {
                            error!("Failed to process job {}: {:?}", job_id, e);
                            let _ = retry_process_job(&worker_pool2,&worker_heap2, job_id).await;
                            // 这个retry，最后有关闭enabled的逻辑，小时间的任务失败了不会自回归，关闭enable不影响，大时间的任务关闭了enable下次reload就不会带着他了
                            // 错误后将任务继续加回。。不需要加回，小时间的重试成功自动加回，重试失败不加回，大时间的重试后等待reload加回，失败后关闭enable避免reload到
                            // let _ = reload_single_job(&worker_pool2, job_id, worker_heap2.clone()).await;
                            
                        }
                    });
                }
                Ok(None) => {
                    debug!("No jobs available");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    error!("Scheduler error: {:?}", e);
                    // 防止 Redis 挂了导致 CPU 空转 100%
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    });

    match signal::ctrl_c().await {
        Ok(()) => info!("Received Ctrl-C, shutting down..."),
        Err(err) => error!("Unable to listen for shutdown signal: {}", err),
    }

    Ok(())
}
