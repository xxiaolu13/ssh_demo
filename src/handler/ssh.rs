use crate::model::ssh::{Client, Message, Session};
use actix_web::{web, HttpResponse};
use actix_web::dev::Server;
use crate::model::http::*;
use actix_web::error::ErrorInternalServerError;
use tracing::log::error;
use crate::model::state::AppState;
use crate::db::server::*;
use crate::utils::crypto::*;
pub async fn ssh_handler(data: web::Data<AppState>,body:web::Json<SshRequest>) -> Result<HttpResponse, actix_web::Error> {
    let id = body.server_id; // 通过server的id确定server。
    let server = get_server_by_id_db(&data.db_pool, id).await.map_err(|e| {
        error!("Failed to get server please register server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to get server please register server")
    })?;
    let msg = Message::new(
        server.ssh_user.clone(),
        passwd_decrypt(server.password.clone()).map_err(|e| {
            error!("Failed to change password: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to change password")
        })?,
        server.ip.clone(),
        server.port.to_string()
    );
    let mut connect = russh::client::connect(msg.config,msg.ip_port,Client).await.map_err(|e| ErrorInternalServerError(e))?;;
    let _ = connect.authenticate_password(msg.user,msg.password).await.map_err(|e| ErrorInternalServerError(e))?;
    println!("Connected to the server");
    let mut ssh = Session{
        session: connect,
    };
    println!("Authentication complete");
    let (code,output) = ssh.call(&body.command).await.map_err(|e| ErrorInternalServerError(e))?;
    ssh.close().await.map_err(|e| ErrorInternalServerError(e))?;
    Ok(HttpResponse::Ok().json(SshResponse {
        exit_code: code,
        output,
    }))
}