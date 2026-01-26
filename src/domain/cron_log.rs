use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct CronLog{
    pub log_id :i32,
    pub job_id :i32,
    pub status :String,
    pub output :Option<String>,
    pub created_at: DateTime<Utc>
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct CreateCronLog{
    pub job_id :i32,
    pub status :String,
    pub output :Option<String>
}

impl CreateCronLog{
    pub fn new(job_id: i32,status: String,output: Option<String>) -> Self{
        Self { job_id, status, output }
    }
}