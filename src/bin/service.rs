use handler::ssh::*;
use actix_web::{web, App, HttpServer,middleware};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use handler::notfound::not_found_handler;
use model::state::AppState;
use crate::handler::server::*;
use crate::handler::servergroup::*;

#[path="../model/mod.rs"]
mod model;
#[path="../handler/mod.rs"]
mod handler;
#[path="../db/mod.rs"]
mod db;
#[path="../utils/mod.rs"]
mod utils;
#[tokio::main]
async fn main()  -> std::io::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL").expect("notfound env var DATABASE_URL");
    info!("Using DATABASE_URL: {}", &db_url);
    let db_pool = PgPoolOptions::new().connect(&db_url).await.unwrap();
    let share_data = web::Data::new(AppState::new(db_pool).await);
    info!("Started http server: 127.0.0.1:8000");
        HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(middleware::NormalizePath::trim())
                .app_data(share_data.clone())
                .service(
                    web::scope("/group")
                        .route("", web::get().to(get_all_groups))// 查找所有的group
                        .route("",web::post().to(create_group))// 创建group
                        .route("/{id}",web::delete().to(delete_group_by_id))// 删除group根据group的id
                        .route("/{id}", web::get().to(get_group_by_id))// 查找group根据group的id
                        .route("/{id}", web::put().to(update_group_by_id))// 更新group信息 根据group的id
                )
                .service(
                    web::scope("/server")
                        .route("",web::get().to(get_all_servers))// 获取所有server
                        .route("",web::post().to(create_single_server))// 创建单个server
                        .route("/group",web::post().to(create_group_server))// 批量创建server
                        .route("/{id}", web::get().to(get_server_by_id))// 根据server的id查找server
                        .route("/{id}",web::delete().to(delete_single_server_by_id))// 删除单个server根据server的id
                        .route("/group/{id}", web::get().to(get_server_by_group_id))// 根据group的id查找server
                )
                .service(
                    web::scope("/ssh")
                        .route("", web::post().to(single_server_ssh_handler))// 单个server执行命令
                        .route("/{id}",web::get().to(test_connect)) // 测试ssh连接
                )
                .default_service(web::route().to(not_found_handler))
        })
            .bind(("localhost", 8080))?
            .run()
            .await
}
