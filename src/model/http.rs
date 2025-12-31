use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SshRequest {
    pub user: Option<String>,      // 可选，默认 root
    pub password: String,
    pub host: String,              // IP地址
    pub port: Option<u16>,         // 可选，默认 22
    pub command: String,           // 要执行的命令
}

#[derive(Debug, Serialize)]
pub struct SshResponse {
    pub exit_code: u32,
    pub output: String,
}