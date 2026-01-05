use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone,Serialize)]
pub struct ServiceTerminal {
    pub id: i32,                // 主键，自增
    pub name: Option<String>,   // 默认None
    pub group_id: Option<i32>,  // 默认None
    pub ssh_user: Option<String>,   // 默认root
    pub ip: String,
    pub port: i32,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct CreateSingleServiceTerminal {
    pub name: Option<String>,   // 默认None
    pub group_id: Option<i32>,  // 默认None
    pub ssh_user: Option<String>,   // 默认root
    pub ip: String,
    pub port: i32,
    pub password: String,
}
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct CreateGroupServiceTerminal { // name为前缀，group必填，ip列表，其他强一致
    pub name_prefix: Option<String>,
    pub group_id: i32,
    pub ssh_user: Option<String>,
    pub ip: Vec<String>,
    pub port: i32,
    pub password: String,
}
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct UpdateServiceTerminal {
    pub name: Option<String>,
    pub group_id: Option<i32>,
    pub ssh_user: Option<String>,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub password: Option<String>,
}

impl TryFrom<web::Json<ServiceTerminal>> for ServiceTerminal {
    type Error = actix_web::Error;
    fn try_from(data: web::Json<ServiceTerminal>) -> Result<Self, Self::Error> {
        Ok(ServiceTerminal{
            id : data.id,
            name: data.name.clone(),
            group_id: data.group_id.clone(),
            ssh_user: data.ssh_user.clone(),
            ip: data.ip.clone(),
            port: data.port,
            password: data.password.clone(),
        })
    }
}
impl TryFrom<web::Json<CreateSingleServiceTerminal>> for CreateSingleServiceTerminal {
    type Error = actix_web::Error;
    fn try_from(data: web::Json<CreateSingleServiceTerminal>) -> Result<Self, Self::Error> {
        Ok(CreateSingleServiceTerminal{
            name: data.name.clone(),
            group_id: data.group_id.clone(),
            ssh_user: data.ssh_user.clone(),
            ip: data.ip.clone(),
            port: data.port,
            password: data.password.clone(),
        })
    }
}
impl TryFrom<web::Json<UpdateServiceTerminal>> for UpdateServiceTerminal {
    type Error = actix_web::Error;
    fn try_from(data: web::Json<UpdateServiceTerminal>) -> Result<Self, Self::Error> {
        Ok(UpdateServiceTerminal{
            name: data.name.clone(),
            group_id: data.group_id.clone(),
            ssh_user: data.ssh_user.clone(),
            ip: data.ip.clone(),
            port: data.port,
            password: data.password.clone(),
        })
    }
}