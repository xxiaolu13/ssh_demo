use crate::model::state::AppState;
use actix_web::{web, HttpResponse};
use crate::model::servergroup::*;
use sqlx::PgPool;
use tracing::log::error;
use crate::db::servergroup::*;
use crate::db::server::get_server_by_group_id_db;

pub async fn get_all_groups(data: web::Data<AppState>) -> Result<HttpResponse, actix_web::Error>{
    let groups = get_all_groups_db(&data.db_pool).await.map_err(|e| {
        error!("Failed to fetch servers: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch servers")
    })?;

    Ok(HttpResponse::Ok().json(groups))
}


pub async fn get_group_by_id(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, actix_web::Error>{
    let group_id = id.into_inner();
    let group = get_group_by_id_db(&data.db_pool, group_id).await.map_err(|e| {
        error!("Failed to fetch server by id: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch servers")
    })?;

    Ok(HttpResponse::Ok().json(group))
}


pub async fn create_group(data: web::Data<AppState>,group: web::Json<CreateGroup>) -> Result<HttpResponse, actix_web::Error> {
    let group = create_group_db(&data.db_pool,group.try_into()?).await.map_err(|e| {
        error!("Failed to create server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch servers")
    })?;
    Ok(HttpResponse::Created().json(group))
}



pub async fn delete_group_by_id(data: web::Data<AppState>,group_id: web::Path<i32>) -> Result<HttpResponse, actix_web::Error> {
    let group_id = group_id.into_inner();
    // 查询下这个group，如果有报错则未查到
    let _ = get_group_by_id_db(&data.db_pool, group_id).await.map_err(|e| {
        error!("Delete group but can't find this group: {:?}", e);
        actix_web::error::ErrorInternalServerError("Delete group but can't find this group")
    })?;
    // 获取这个group下的server
    let server = get_server_by_group_id_db(&data.db_pool, group_id).await.map_err(|e| {
        error!("Delete group Failed to get server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Delete group Failed to get server")
    })?;
    // 判断是否有server，如果没有则删除，有则报错
    let ans = match server.len() {
        0 => delete_group_by_id_db(&data.db_pool,group_id).await.map_err(|e| {
            error!("can't delete this group: {:?}", e);
            actix_web::error::ErrorInternalServerError("can't delete this group")
        })?,
        _ => Err(actix_web::error::ErrorInternalServerError("the group have server"))?,
    };
    Ok(HttpResponse::Ok().json(ans))
}


pub async fn update_group_by_id(data: web::Data<AppState>,group_id: web::Path<i32>,newgroup: web::Json<UpdateGroup>) -> Result<HttpResponse, actix_web::Error> {
    let group_id = group_id.into_inner();
    let group = update_group_by_id_db(&data.db_pool, group_id,newgroup.try_into()?).await.map_err(|e| {
        error!("can't update this group: {:?}", e);
        actix_web::error::ErrorInternalServerError("can't update this group")
    })?;
    Ok(HttpResponse::Ok().json(group))
}