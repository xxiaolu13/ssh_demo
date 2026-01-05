use handler::ssh::ssh_handler;
use actix_web::{web, App, HttpServer,middleware};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use handler::notfound::not_found_handler;
use model::state::AppState;
use crate::handler::server::{create_single_server, get_all_servers, get_server_by_group_id, get_server_by_id};
use crate::handler::servergroup::{create_group, delete_group_by_id, get_all_groups, get_group_by_id};

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
                        .route("", web::get().to(get_all_groups))
                        .route("",web::post().to(create_group))
                        .route("/{id}",web::delete().to(delete_group_by_id))
                        .route("/{id}", web::get().to(get_group_by_id))
                )
                .service(
                    web::scope("/server")
                        .route("",web::get().to(get_all_servers))
                        .route("",web::post().to(create_single_server))
                        .route("/{id}", web::get().to(get_server_by_id))
                        .route("/group/{id}", web::get().to(get_server_by_group_id))
                )
                .service(
                    web::scope("/ssh")
                        .route("", web::post().to(ssh_handler))
                )
                .default_service(web::route().to(not_found_handler))
        })
            .bind(("localhost", 8080))?
            .run()
            .await
}
