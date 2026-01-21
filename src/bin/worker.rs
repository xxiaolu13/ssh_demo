use dotenvy::dotenv;
use sqlx::PgPool;
use connect_ok::domain::scheduler::JobScheduler;
use connect_ok::scheduler::prepare::reload_job_from_sql;
use tracing::{info, debug, error};
use tokio::signal;
use connect_ok::repository::cron_job::*;
use connect_ok::scheduler::prepare::*;
use anyhow::Result;

// 业务逻辑抽离出来
async fn process_job(pool: &PgPool, heap: &JobScheduler, job_id: i32) -> Result<()> {
    heap.del_job(job_id).await?; 
    info!("job {} start execute", job_id);

    let msg = get_cronjob_by_id_db(pool, job_id).await?;
    match msg.group_id {
        Some(_) => {
            batch_job_execute(pool, msg.clone()).await?;     
        },
        None => {
            single_job_execute(pool, msg.clone()).await?; 
        }
    }
    reload_single_job(pool, msg.id, heap.clone()).await?;
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
    
    let sec = std::env::var("RELOAD_SECS")
    .unwrap_or("3600".to_string()).parse().expect("RELOAD_SECS must be number");
    info!("Worker reloads once every {} secs", &sec);
    // 初始化加载
    let pool1 = pool.clone();
    let heap1 = heap.clone();
    // 首次运行 先reload next execute at,如果不这么做，在执行时候，worker会有任务补偿，将所有任务都执行一遍
    let _ = reload_job_from_sql(&pool, heap.clone()).await?;

    // 定时轮询数据库
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(sec));
        interval.tick().await; 
        loop {
            interval.tick().await;
            match reload_job_from_sql(&pool1, heap1.clone()).await {
                Ok(_) => info!("Reload job from sql success.."),
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
                            // 可选：如果是严重错误，可能需要在这里做一些补偿逻辑
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
