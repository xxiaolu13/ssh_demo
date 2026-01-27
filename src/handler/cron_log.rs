use actix_web::{HttpResponse, web};
use crate::db::pool::AppState;
use crate::repository::cron_log::get_cron_log_by_job_id_db;
use log::error;





// pub async fn create_cron_log(data: web::Data<AppState>,params: web::Json<CreateCronLog>) -> Result<HttpResponse, actix_web::Error> {
//     let createcronlog = params.into_inner();

//     let row = create_cron_log_db(&data.db_pool, createcronlog).await.map_err(|e| {
//         error!("Failed to create a cronlog: {:?}", e);
//         actix_web::error::ErrorInternalServerError("Failed to create a cronlog")})?;

//     Ok(HttpResponse::Ok().json(row))
// }


pub async fn get_cron_log_by_job_id(data: web::Data<AppState>,job_id: web::Path<i32>) -> Result<HttpResponse, actix_web::Error>{
    let job_id = job_id.into_inner();
    let row = get_cron_log_by_job_id_db(&data.db_pool, job_id).await.map_err(|e| {
        error!("Failed to get cronlog: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to get cronlog")})?;
    Ok(HttpResponse::Ok().json(row))
}