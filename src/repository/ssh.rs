use std::sync::Arc;
use crate::{domain::ssh_configuration::{Client, Message, Session}};
use crate::domain::ssh_session::*;
use actix_web::error::ErrorInternalServerError;
use tracing::log::{info,warn};
use crate::utils::crypto::*;
use actix_web::error::{ErrorPayloadTooLarge,ErrorRequestTimeout,ErrorGatewayTimeout};
use tokio::time::{Duration, timeout};
use std::env;
use bytes::Bytes;




// 超时问题
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
const AUTH_TIMEOUT: Duration = Duration::from_secs(5);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(15);

pub async fn test_connect_back(msg: Message) -> Result<String, actix_web::Error> {
    let ip = msg.ipaddr.unwrap_or("".to_string());
    let ip_port = format!("{}:{}",ip,msg.port);
    let mut connect: russh::client::Handle<Client> = timeout(
        CONNECTION_TIMEOUT,
        russh::client::connect(msg.config.clone(), ip_port.clone(), Client)
    )
    .await
    .map_err(|_| ErrorRequestTimeout("Connection timeout"))?  // 处理 timeout 错误
    .map_err(|e| ErrorInternalServerError(e))?;               // 处理 russh 错误

    let password = passwd_decrypt(msg.password.clone())
        .map_err(|e| ErrorInternalServerError(format!("Password decryption failed: {}", e)))?;

    timeout(
        AUTH_TIMEOUT,
        connect.authenticate_password(msg.user, password)
    )
    .await
    .map_err(|_| ErrorGatewayTimeout("Auth timeout"))?
    .map_err(|e| ErrorInternalServerError(format!("Authentication failed: {}", e)))?;

    connect.disconnect(russh::Disconnect::ByApplication, "", "en").await
        .ok();
    Ok::<String, actix_web::Error>(format!("connected to server {} successfully", ip_port))
}



pub async fn single_server_ssh_back(msg: Message,command: String) -> Result<(u32,String), actix_web::Error> {
    let ip = msg.ipaddr.unwrap_or("".to_string());
    let ip_port = format!("{}:{}",ip,msg.port);
    info!("connect to {}",ip_port);
    let password = passwd_decrypt(msg.password.clone())
        .map_err(|e| ErrorInternalServerError(format!("Password decryption failed: {}", e)))?;
    let mut connect: russh::client::Handle<Client> = timeout(
        CONNECTION_TIMEOUT,
        russh::client::connect(msg.config,ip_port,Client)
    )
    .await
    .map_err(|_| ErrorRequestTimeout("Connection timeout"))?  // 处理 timeout 错误
    .map_err(|e| ErrorInternalServerError(e))?;               // 处理 russh 错误

    timeout(
        AUTH_TIMEOUT,
        connect.authenticate_password(msg.user,password)
    )
    .await
    .map_err(|_| ErrorGatewayTimeout("Auth timeout"))?
    .map_err(|e| ErrorInternalServerError(format!("Authentication failed: {}", e)))?;


    info!("Connected to the server");
    let mut ssh = Session{
        session: connect,
    };
    info!("Authentication complete");
    let (code, output) = timeout(
        COMMAND_TIMEOUT,
        ssh.call(&command)
    )
    .await
    .map_err(|_| ErrorGatewayTimeout("Command timeout"))?
    .map_err(|e| ErrorInternalServerError(e))?;
    info!("timeout success");
    // let (code,output) = ssh.call(&body.command).await.map_err(|e| ErrorInternalServerError(e))?;
    const MAX_OUTPUT_SIZE: usize = 1 * 1024 * 1024; // 1MB

    // 检查输出大小
    if output.len() > MAX_OUTPUT_SIZE {
        return Err(ErrorPayloadTooLarge(
            format!("Output exceeds {}MB limit", MAX_OUTPUT_SIZE / 1024 / 1024)
        ));
    }
    info!("output success");
    ssh.close().await.map_err(|e| ErrorInternalServerError(e))?;
    info!("ssh closed");
    Ok((code, output))
}



pub async fn batch_server_ssh_back(msg: Message,command: String) -> Result<tokio::sync::mpsc::Receiver<Result<Bytes, std::io::Error>>, actix_web::Error> {
    let server_list = msg.server_list.unwrap_or(Vec::new());
    // 异步
    let buffer_size = env::var("CNOK_CHANNEL_BUFFER")
    .ok()
    .and_then(|s| s.parse().ok())
    .unwrap_or(100); // 默认值
    info!("channel buffer is {}",buffer_size);
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(buffer_size);
    let config = Arc::new(russh::client::Config::default());
    let command = Arc::new(command.clone());
    for server in server_list{
            let tx = tx.clone();
            let server_label = server.clone();
            let config = Arc::clone(&config);
            let command = Arc::clone(&command);
            let ip_port = format!("{}:{}",server,msg.port);
            let user = msg.user.clone();
            let password = msg.password.clone();

        tokio::spawn(async move{
            let result = batch_ssh_execute(
                config, 
                ip_port.clone(), 
                user, 
                password, 
                command  // 直接传递 Arc<String>
            ).await;
            
            let final_json = match result {
                Ok(output) => {
                    let ssh_result = SshResult {
                        server: server_label.clone(),
                        output,
                        exit_code: Some(0),
                    };
                    let back = serde_json::to_string(&ssh_result).unwrap_or_else(|_| {
                         format!(r#"{{"server":"{}","error":"JSON serialization failed"}}"#, server_label)
                    });
                    info!("batch server: {} done",server_label.clone());
                    back
                    
                    
                },
                Err(e) => {
                    let error_result = SshError {
                        server: server_label.clone(),
                        output: e.to_string(),
                        exit_code: Some(1),
                    };
                    let back = serde_json::to_string(&error_result).unwrap_or_else(|_| {
                        format!(r#"{{"server":"{}","error":"{}"}}"#, server_label, e)
                    });
                    warn!("batch server: {}",back);
                    back
                }
            };
            
            let json_with_newline = format!("{}\n", final_json);
            // 转换为 Bytes 并发送
            // 这里的 Error 类型必须是 std::io::Error (或者你在 Channel 定义的那个)
            if let Err(_closed) = tx.send(Ok(Bytes::from(json_with_newline))).await {
                // Channel 已关闭，通常不需要处理，或者记录日志
                warn!("Receiver dropped");
            }
        });
    }
    drop(tx); // 关闭主线程的发送端，否则接收端永远不会结束
    Ok(rx)

}

// 防止batch server ssh handler中tokio spawn中的嵌套，所以单独拿出来这部分，后续加密钥认证方便改
async fn batch_ssh_execute(
    config: Arc<russh::client::Config>,
    ip_port: String,
    user: String,
    password: String,
    command: Arc<String>
) -> Result<String, String> {
    let ip_port_clone = ip_port.clone();
    let mut connect: russh::client::Handle<Client> = timeout(
        CONNECTION_TIMEOUT,
        russh::client::connect(config, ip_port, Client)
    )
    .await
    .map_err(|_| format!("Connection timeout to {}", ip_port_clone))?
    .map_err(|e| format!("Connection failed: {}", e))?;              // 处理 russh 错误
    let user_clone = user.clone();
    // 认证
    timeout(
        AUTH_TIMEOUT,
        connect.authenticate_password(user,password)
    )
    .await
    .map_err(|_| format!("Auth timeout for user {}", user_clone))?
    .map_err(|e| format!("Authentication failed: {}", e))?;


    info!("Connected to the server");
    let mut ssh = Session{
        session: connect,
    };
    info!("Authentication complete");

    let (_code, output) = timeout(
        COMMAND_TIMEOUT,
        ssh.call(command.as_str())
    )
    .await
    .map_err(|_| "Command execution timeout".to_string())?
    .map_err(|e| format!("Command execution failed: {}", e))?;

    // let (code,output) = ssh.call(&body.command).await.map_err(|e| ErrorInternalServerError(e))?;
    const MAX_OUTPUT_SIZE: usize = 1 * 1024 * 1024; // 1MB

    // 检查输出大小
    if output.len() > MAX_OUTPUT_SIZE {
        return Err(format!(
            "Output exceeds {}MB limit (actual: {}MB)", 
            MAX_OUTPUT_SIZE / 1024 / 1024,
            output.len() / 1024 / 1024
        ));
    }

    ssh.close().await.map_err(|e| format!("Failed to close connection: {}", e))?;
    
    Ok(output)
}