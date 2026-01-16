use actix_web::{web,HttpResponse};
use log::error;
use tracing::field::debug;
use crate::db::pool::AppState;
use crate::domain::cron_job::{CreateCronJob, UpdateCronJob};
use crate::repository::cron_job::{get_all_cronjobs_db, get_cronjob_by_id_db,create_cronjob_db,update_cronjob_db};

pub async fn get_all_cronjobs(data:web::Data<AppState>) -> Result<HttpResponse, actix_web::Error>{
    let rows = get_all_cronjobs_db(&data.db_pool).await.map_err(|e| {
        error!("Failed to get cronjobs: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to get cronjobs")})?;
    Ok(HttpResponse::Ok().json(rows))
}


pub async fn get_cronjob_by_id(data: web::Data<AppState>,params: web::Path<i32>) -> Result<HttpResponse, actix_web::Error> {
    let id = params.into_inner();
    let row  =  get_cronjob_by_id_db(&data.db_pool, id).await.map_err(|e| {
        error!("Failed to get a cronjob: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to get a cronjob")
    })?;
    Ok(HttpResponse::Ok().json(row))
}



pub async fn create_cronjob(data: web::Data<AppState>,job: web::Json<CreateCronJob>) -> Result<HttpResponse, actix_web::Error> {
    debug("test cron job handler started");
    let row = create_cronjob_db(&data.db_pool, job.into_inner().try_into()?).await.map_err(|e| {
        error!("Failed to create a cronjob: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to create a cronjob")})?;
    Ok(HttpResponse::Ok().json(row))
}


pub async fn update_cronjob(data: web::Data<AppState>,job_id:web::Path<i32>,job: web::Json<UpdateCronJob>) -> Result<HttpResponse, actix_web::Error> {
    let row = update_cronjob_db(&data.db_pool, job_id.into_inner(),job.into_inner().try_into()?).await.map_err(|e| {
        error!("Failed to update cronjob: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to update cronjob")})?;
    Ok(HttpResponse::Ok().json(row))
}















