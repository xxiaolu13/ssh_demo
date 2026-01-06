use crate::model::state::AppState;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use tracing::log::error;
use crate::db::server::*;
use crate::db::servergroup::create_group_db;
use crate::model::server::*;
use crate::model::servergroup::*;

// update的暂时不写了

pub async fn get_all_servers(
    data: web::Data<AppState>
) -> Result<HttpResponse, actix_web::Error> {
    let servers = get_all_servers_db(&data.db_pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch servers: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch servers")
        })?;
    Ok(HttpResponse::Ok().json(servers))
}

pub async fn get_server_by_id(
    data: web::Data<AppState>,
    params: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let server_id = params.into_inner();
    let server = get_server_by_id_db(&data.db_pool,server_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch servers: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch servers")
        })?;
    Ok(HttpResponse::Ok().json(server))
}

pub async fn get_server_by_group_id(
    data: web::Data<AppState>,
    group_id: web::Path<i32>,
) -> Result<HttpResponse, actix_web::Error> {
    let group_id = group_id.into_inner();
    let server = get_server_by_group_id_db(&data.db_pool,group_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch servers: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch servers")
        })?;
    Ok(HttpResponse::Ok().json(server))
}

pub async fn create_single_server(data: web::Data<AppState>,server: web::Json<CreateSingleServiceTerminal>) -> Result<HttpResponse, actix_web::Error> {
    let server = create_single_server_db(&data.db_pool,server.try_into()?).await.map_err(|e| {
        error!("Failed to create a server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to create a server")
    })?;
    Ok(HttpResponse::Ok().json(server))
}

pub async fn create_group_server(data: web::Data<AppState>,server_list: web::Json<CreateGroupServiceTerminal>) -> Result<HttpResponse, actix_web::Error> {
    let server_list  = server_list.into_inner().try_into()?;
    let ans = create_group_server_db(&data.db_pool,server_list).await.map_err(|e| {
        error!("Failed to create a server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to create a server")
    })?;
    Ok(HttpResponse::Ok().json(ans))
}


pub async fn delete_single_server_by_id(data: web::Data<AppState>,server_id: web::Path<i32>) -> Result<HttpResponse, actix_web::Error> {
    let server_id = server_id.into_inner();
    let _ = get_server_by_id_db(&data.db_pool,server_id)
        .await
        .map_err(|e| {
            error!("Delete server Failed to fetch server: {:?}", e);
            actix_web::error::ErrorInternalServerError("Delete server Failed to fetch server")
        })?;
    let ans = delete_single_server_by_id_db(&data.db_pool,server_id).await
        .map_err(|e| {
        error!("Failed to Delete server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to Delete server")
    })?;
    Ok(HttpResponse::Ok().json(ans))
}

