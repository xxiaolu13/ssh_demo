use crate::{domain::ssh_configuration::Message, repository::ssh::{test_connect_back,single_server_ssh_back,batch_server_ssh_back}};
use actix_web::{web, HttpResponse};
use crate::domain::ssh_session::*;
use tracing::log::error;
use crate::db::pool::AppState;
use crate::repository::server::*;
use crate::utils::crypto::*;
use crate::domain::server::ServiceTerminal;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;





pub async fn test_connect_handler(data: web::Data<AppState>,server_id: web::Path<i32>) -> Result<HttpResponse, actix_web::Error> {
    let server_id = server_id.into_inner();
    let server = get_server_by_id_db(&data.db_pool, server_id).await.map_err(|e| {
        error!("Failed to get server please register server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to get server please register server")
    })?;
    let msg = Message::new(server.ssh_user, server.password.clone(),server.port.to_string(), Some(server.ip),None);
    let result = test_connect_back(msg).await?;
    Ok(HttpResponse::Ok().body(result))
}



pub async fn single_server_ssh_handler(data: web::Data<AppState>,body:web::Json<SshRequest>) -> Result<HttpResponse, actix_web::Error> {
    let id = body.server_id; // 通过server的id确定server。
    let server = get_server_by_id_db(&data.db_pool, id).await.map_err(|e| {
        error!("Failed to get server please register server: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to get server please register server")
    })?;
    let msg = Message::new(server.ssh_user, server.password.clone(),server.port.to_string(), Some(server.ip),None);
    let (code,output) = single_server_ssh_back(None,&data.db_pool,msg, body.command.clone()).await?;
    Ok(HttpResponse::Ok().json(SshResponse {
        exit_code: code,
        output,
    }))
}



pub async fn batch_server_ssh_handler(data: web::Data<AppState>,body:web::Json<BatchSshRequest>) -> Result<HttpResponse, actix_web::Error> {
    // 处理server信息，获取地址的vec
    let body = body.into_inner();
    let group_id = body.group_id;
    let server_list:Vec<ServiceTerminal> = get_server_by_group_id_db(&data.db_pool, group_id).await.map_err(|e| {
        error!("Failed to get server by group_id: {:?}", e);
        actix_web::error::ErrorInternalServerError("Failed to get server by group_id")
    })?;

    let ssh_user = server_list[0].ssh_user.clone();
    let password = passwd_decrypt(server_list[0].password.clone()).map_err(|e| {
            error!("Failed to change password: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to change password")
        })?;
    let port = server_list[0].port.to_string();
    let server_list:Vec<String> = server_list.into_iter().map(|e|e.ip.clone()).collect();
    
    let msg = Message::new(ssh_user, password, port, None, Some(server_list));

    let rx = batch_server_ssh_back(None,&data.db_pool,msg, body.command.clone()).await?;

    // 异步
    // let buffer_size = env::var("CNOK_CHANNEL_BUFFER")
    // .ok()
    // .and_then(|s| s.parse().ok())
    // .unwrap_or(100); // 默认值
    // info!("channel buffer is {}",buffer_size);
    // let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(buffer_size);
    // let config = Arc::new(russh::client::Config::default());
    // let command = Arc::new(body.command.clone());
    // for server in server_list{
    //         let tx = tx.clone();
    //         let server_label = server.clone();
    //         let config = Arc::clone(&config);
    //         let command = Arc::clone(&command);
    //         let ip_port = format!("{}:{}",server,port);
    //         let user = ssh_user.clone();
    //         let password = password.clone();

    //     tokio::spawn(async move{
    //         let result = batch_ssh_execute(
    //             config, 
    //             ip_port.clone(), 
    //             user, 
    //             password, 
    //             command  // 直接传递 Arc<String>
    //         ).await;
            
    //         let final_json = match result {
    //             Ok(output) => {
    //                 let ssh_result = SshResult {
    //                     server: server_label.clone(),
    //                     output,
    //                     exit_code: Some(0),
    //                 };
    //                 let back = serde_json::to_string(&ssh_result).unwrap_or_else(|_| {
    //                      format!(r#"{{"server":"{}","error":"JSON serialization failed"}}"#, server_label)
    //                 });
    //                 info!("batch server: {} done",server_label.clone());
    //                 back
                    
                    
    //             },
    //             Err(e) => {
    //                 let error_result = SshError {
    //                     server: server_label.clone(),
    //                     output: e.to_string(),
    //                     exit_code: Some(1),
    //                 };
    //                 let back = serde_json::to_string(&error_result).unwrap_or_else(|_| {
    //                     format!(r#"{{"server":"{}","error":"{}"}}"#, server_label, e)
    //                 });
    //                 warn!("batch server: {}",back);
    //                 back
    //             }
    //         };
            
    //         let json_with_newline = format!("{}\n", final_json);
    //         // 转换为 Bytes 并发送
    //         // 这里的 Error 类型必须是 std::io::Error (或者你在 Channel 定义的那个)
    //         if let Err(_closed) = tx.send(Ok(Bytes::from(json_with_newline))).await {
    //             // Channel 已关闭，通常不需要处理，或者记录日志
    //             warn!("Receiver dropped"); 
    //         }
    //     });
    // }
    // drop(tx); // 关闭主线程的发送端，否则接收端永远不会结束

    // 将接收流映射为 Actix 需要的 Result<Bytes, actix_web::Error>
    let stream = ReceiverStream::new(rx).map(|res| {
        // 将 std::io::Error 映射为 actix_web::Error
        res.map_err(|e| actix_web::error::ErrorInternalServerError(e))
    });

    Ok(HttpResponse::Ok()
            .content_type("application/x-ndjson") // application/x-ndjson
            .streaming(stream))

}
