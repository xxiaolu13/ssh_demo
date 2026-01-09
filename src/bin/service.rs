use handler::ssh::*;
use actix_web::{web, App, HttpServer,middleware};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use handler::notfound::not_found_handler;
use model::state::AppState;
use crate::handler::server::*;
use crate::handler::servergroup::*;
use actix_cors::Cors;
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

    info!("Started http server");
        HttpServer::new(move || {
                let cors = Cors::default()
                .allow_any_origin()  // 允许任何来源（开发环境）
                .allow_any_method()  // 允许所有 HTTP 方法
                .allow_any_header()  // 允许所有请求头
                .max_age(3600);      // 预检请求缓存时间
            App::new()
                .wrap(cors)
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
                        .route("/batch",web::post().to(batch_server_ssh_handler))
                        .route("/{id}",web::get().to(test_connect)) // 测试ssh连接
                )
                .default_service(web::route().to(not_found_handler))
        })
            .bind(("0.0.0.0", 8080))?
            .run()
            .await
}
