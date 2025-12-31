use handler::ssh::ssh_handler;
use actix_web::{web, App, HttpServer,middleware};
use handler::notfound::not_found_handler;
#[path="../model/mod.rs"]
mod model;
#[path="../handler/mod.rs"]
mod handler;



#[tokio::main]
async fn main()  -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
        HttpServer::new( || {
            App::new()
                .wrap(middleware::NormalizePath::trim()) // 移除末尾的 /
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
