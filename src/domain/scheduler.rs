use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use anyhow::anyhow;
use redis::{Client, AsyncCommands, Script};
use std::env;
use tracing::info;

const ACQUIRE_JOB_SCRIPT: &str = include_str!("../script/acquire_job.lua");
#[derive(Debug,Eq,PartialEq,PartialOrd)]
pub struct CronWorker {
    pub next_execute_at: DateTime<Utc>,
    pub cronjob_id: i32
}
pub struct SchedulerInner{ // 小顶堆
    heap: BinaryHeap<Reverse<CronWorker>>
}
pub struct Scheduler{
    inner: Arc<Mutex<SchedulerInner>>,
}
impl Ord for CronWorker {
    fn cmp(&self, other: &Self) -> Ordering {
        self.next_execute_at.cmp(&other.next_execute_at)
    }
}
impl CronWorker{
    pub fn new(next_execute_at: DateTime<Utc>, cronjob_id: i32) -> Self{
        CronWorker{
            next_execute_at,
            cronjob_id
        }
    }
}
impl Scheduler{
    pub fn new() -> Self{
        Scheduler{
            inner: Arc::new(Mutex::new(
                SchedulerInner{
                    heap: BinaryHeap::new(),
                }
            ))
        }
    }
    pub fn push(&self, worker: CronWorker) -> Result<(),anyhow::Error> {
        let mut lock = self.inner.lock().map_err(|e| anyhow!(e.to_string()))?;
        Ok(lock.heap.push(Reverse(worker)))
    }
    pub fn pop(&self) -> Result<Option<Reverse<CronWorker>>, anyhow::Error> {
        let lock = self.inner.lock();
        match lock{
            Ok(mut worker) => {
                Ok(worker.heap.pop())
            }
            _ => Err(anyhow::Error::msg("can't get mutex lock"))
        }
    }
}

#[derive(Debug,Clone)]
pub struct JobScheduler {
    redis: Client,
    script_sha: String,
}
impl JobScheduler {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://:xiaolu@127.0.0.1:6379/".to_string());

        let client = Client::open(redis_url)?;
        let mut con = client.get_multiplexed_async_connection().await?;
        // 预加载脚本
        let sha: String = Script::new(ACQUIRE_JOB_SCRIPT)
            .prepare_invoke()
            .load_async(&mut con)
            .await?;

        Ok(Self {
            redis: client,
            script_sha: sha,
        })
    }
    /// 从待执行队列获取一个到期任务
    pub async fn get_job(&self) -> Result<Option<i32>, anyhow::Error> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;

        let current_ts = Utc::now().timestamp_millis();
        let timeout_ms = 10_000;  // 30秒超时

        let job_id: Option<i32> = redis::cmd("EVALSHA")
            .arg(&self.script_sha)
            .arg(2)  // 2 个 KEYS
            .arg("scheduler:pending")
            .arg("scheduler:processing")
            .arg(current_ts)
            .arg(timeout_ms)
            .query_async(&mut con)
            .await?;

        Ok(job_id)
    }

    /// 添加任务到待执行队列
    pub async fn add_job(
        &self,
        job_id: i32,
        execute_at: i64  // 毫秒时间戳
    ) -> Result<(), anyhow::Error> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;

        // ZADD scheduler:pending <timestamp> <job_id>
        let _:() = con.zadd("scheduler:pending", job_id, execute_at).await?;
        info!("job {} added to queue", job_id);

        Ok(())
    }
    /// 任务完成，从处理队列移除
    pub async fn del_job(&self, job_id: i32) -> Result<(), anyhow::Error> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;

        let _:() =con.zrem("scheduler:processing", job_id).await?;

        Ok(())
    }

    /// 任务失败，重新放回待执行队列
    pub async fn retry_job(
        &self,
        job_id: i32,
        retry_after: i64  // 毫秒时间戳
    ) -> Result<(), anyhow::Error> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;

        // 从处理中移除
        let _:() =con.zrem("scheduler:processing", job_id).await?;

        // 重新加入待执行队列
        let _:() =con.zadd("scheduler:pending", job_id, retry_after).await?;

        Ok(())
    }

    /// 清理超时任务（容错机制）
    pub async fn del_timeout_jobs(&self) -> Result<Vec<i32>, anyhow::Error> {
        let mut con = self.redis.get_multiplexed_async_connection().await?;

        let current_ts = Utc::now().timestamp_millis();

        // 查询超时的任务（deadline < now）
        let timeout_jobs: Vec<i32> = con.zrangebyscore(
            "scheduler:processing",
            "-inf",
            current_ts
        ).await?;
        // 将超时任务移回待执行队列
        for job_id in &timeout_jobs {
            let _:() =con.zrem("scheduler:processing", job_id).await?;

            // 立即重试或延迟重试
            let retry_at = current_ts + 5000;  // 5秒后重试
            let _:() =con.zadd("scheduler:pending", job_id, retry_at).await?;
        }

        Ok(timeout_jobs)
    }
}