
use dotenvy::dotenv;
use sqlx::PgPool;
use connect_ok::domain::scheduler::JobScheduler;
use connect_ok::scheduler::prepare::reload_job_from_sql;
use tracing::{info, debug,error};
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    info!("Process started with PID: {}", std::process::id());
    let db_url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&db_url).await?;

    let heap = JobScheduler::new().await?;

    let _ = reload_job_from_sql(&pool, heap.clone()).await?;
    tokio::spawn(async move {
        loop {
            match heap.get_job().await {
                Ok(Some(job_id)) => {
                    info!("job {} added to queue", job_id);

                    match heap.del_job(job_id).await {
                        Ok(_) => {
                            info!("job {} completed", job_id)
                            // todo()!
                        },
                        Err(e) => error!("job {} failed: {}", job_id, e),
                    }
                }
                Ok(None) => {
                    debug!("No jobs available");
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Failed to acquire job: {}", e);
                }
            }
        }
    });









    Ok(())
}