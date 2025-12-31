use crate::model::ssh::{Client, Message, Session};
use actix_web::{web, HttpResponse};
use crate::model::http::*;
use actix_web::error::ErrorInternalServerError;
pub async fn ssh_handler(body:web::Json<SshRequest>) -> Result<HttpResponse, actix_web::Error> {
    let user = body.user.as_deref().unwrap_or("root");
    let port = body.port.unwrap_or(22);
    let msg = Message::new(user.to_string(),body.password.clone(),body.host.clone(),port.to_string());
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