use actix_web::web;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Local, Utc};
use sqlx::FromRow;
use cron_parser::parse;
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CronJob {
    pub id: i32,
    pub name: Option<String>,
    pub cron_expression: String,
    pub server_id: Option<i32>,
    pub group_id: Option<i32>,
    pub command: String,
    pub enabled: bool,
    pub timeout: Option<i32>,
    pub retry_count: Option<i32>,
    pub description: Option<String>,
    pub last_executed_at: Option<DateTime<Utc>>,
    pub next_execute_at: DateTime<Utc>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl TryFrom<web::Json<CronJob>> for CronJob {
    type Error = actix_web::Error;
    fn try_from(json: web::Json<CronJob>) -> actix_web::Result<CronJob, Self::Error> {
        Ok(CronJob{
            id : json.id,
            name: json.name.clone(),
            cron_expression: json.cron_expression.clone(),
            server_id: json.server_id,
            group_id: json.group_id,
            command: json.command.clone(),
            enabled: json.enabled,
            timeout: json.timeout,
            retry_count: json.retry_count,
            description: json.description.clone(),
            last_executed_at: json.last_executed_at.clone(),
            next_execute_at: json.next_execute_at.clone(),
            created_at: json.created_at.clone(),
            updated_at: json.updated_at.clone()
        })
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateCronJob {
    pub name: Option<String>,
    pub cron_expression: String,
    pub server_id: Option<i32>,
    pub group_id: Option<i32>,
    pub command: String,
    pub enabled: bool,
    pub timeout: Option<i32>,
    pub retry_count: Option<i32>,
    pub description: Option<String>,
    #[serde(skip_deserializing)]
    pub next_execute_at: DateTime<Utc>,
}
impl TryFrom<web::Json<CreateCronJob>> for CreateCronJob {
    type Error = actix_web::Error;
    fn try_from(json: web::Json<CreateCronJob>) -> actix_web::Result<CreateCronJob, Self::Error> {
        Ok(CreateCronJob{
            name: json.name.clone(),
            cron_expression: json.cron_expression.clone(),
            server_id: json.server_id,
            group_id: json.group_id,
            command: json.command.clone(),
            enabled: json.enabled,
            timeout: json.timeout,
            retry_count: json.retry_count,
            description: json.description.clone(),
            next_execute_at: json.next_execute_at.clone(),
        })
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateCronJob {
    pub name: Option<String>,
    pub cron_expression: Option<String>,
    pub server_id: Option<i32>,
    pub group_id: Option<i32>,
    pub command: Option<String>,
    pub enabled: Option<bool>,
    pub timeout: Option<i32>,
    pub retry_count: Option<i32>,
    pub description: Option<String>,
    #[serde(skip_deserializing)]
    pub next_execute_at: Option<DateTime<Utc>>,
}

impl TryFrom<web::Json<UpdateCronJob>> for UpdateCronJob {
    type Error = actix_web::Error;
    fn try_from(json: web::Json<UpdateCronJob>) -> actix_web::Result<UpdateCronJob, Self::Error> {
        Ok(UpdateCronJob{
            name: json.name.clone(),
            cron_expression: json.cron_expression.clone(),
            server_id: json.server_id,
            group_id: json.group_id,
            command: json.command.clone(),
            enabled: json.enabled,
            timeout: json.timeout,
            retry_count: json.retry_count,
            description: json.description.clone(),
            next_execute_at: json.next_execute_at.clone(),

        })
    }
}




pub trait CronJobExecutor {
    fn get_cron_expression(&self) -> &str;
    fn next_tick(&self) ->Result<DateTime<Utc>,anyhow::Error> {
        let time = parse(self.get_cron_expression(), &Local::now())
        .map_err(|e| anyhow!("Invalid cron expression: {}", e))?;
        Ok(time.with_timezone(&Utc))
    }
}

impl CronJobExecutor for CronJob {
    fn get_cron_expression(&self) -> &str {
        &self.cron_expression
    }
}

impl CronJobExecutor for CreateCronJob {
    fn get_cron_expression(&self) -> &str {
        &self.cron_expression
    }
}