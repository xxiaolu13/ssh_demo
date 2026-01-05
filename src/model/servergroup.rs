use actix_web::web;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
#[derive(Deserialize, Debug, Clone,Serialize,sqlx::FromRow)]
pub struct Group{
    pub group_id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    // pub created_at: DateTime<Utc>,
    // pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Deserialize,Serialize)]
pub struct CreateGroup {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug, Clone,Serialize)]
pub struct UpdateGroup{
    pub name: Option<String>,
    pub description: Option<String>,
}

impl TryFrom<web::Json<Group>> for Group {
    type Error = actix_web::Error;
    fn try_from(data: web::Json<Group>) -> actix_web::Result<Group> {
        Ok(
            Group {
                group_id: data.group_id,
                name: data.name.clone(),
                description: data.description.clone(),
                // created_at: data.created_at,
                // updated_at: data.updated_at
            })
    }
}
impl TryFrom<web::Json<CreateGroup>> for CreateGroup {
    type Error = actix_web::Error;
    fn try_from(data: web::Json<CreateGroup>) -> actix_web::Result<CreateGroup> {
        Ok(
            CreateGroup {
                name: data.name.clone(),
                description: data.description.clone()
            })
    }
}
impl TryFrom<web::Json<UpdateGroup>> for UpdateGroup {
    type Error = actix_web::Error;
    fn try_from(data: web::Json<UpdateGroup>) -> actix_web::Result<UpdateGroup> {
        Ok(
            UpdateGroup {
                name: data.name.clone(),
                description: data.description.clone()
            })
    }
}