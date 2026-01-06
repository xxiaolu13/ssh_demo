use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SshRequest {
    pub server_id: i32,
    pub command: String,           // 要执行的命令
}

#[derive(Debug, Serialize)]
pub struct SshResponse {
    pub exit_code: u32,
    pub output: String,
}