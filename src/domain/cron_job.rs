use actix_web::web;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CronJob {
    // id 自增
    pub id: i32,
    pub name: Option<String>,
    pub cron_expression: String, // 表达式
    pub server_id: Option<i32>,
    pub group_id: Option<i32>,
    pub command: String, // 执行的任务
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
            command: json.command.clone()
        })
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateCronJob {
    pub name: Option<String>,
    pub cron_expression: String, // 表达式
    pub server_id: Option<i32>,
    pub group_id: Option<i32>,
    pub command: String, // 执行的任务
}
impl TryFrom<web::Json<CreateCronJob>> for CreateCronJob {
    type Error = actix_web::Error;
    fn try_from(json: web::Json<CreateCronJob>) -> actix_web::Result<CreateCronJob, Self::Error> {
        Ok(CreateCronJob{
            name: json.name.clone(),
            cron_expression: json.cron_expression.clone(),
            server_id: json.server_id,
            group_id: json.group_id,
            command: json.command.clone()
        })
    }
}

