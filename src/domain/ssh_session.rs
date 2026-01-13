use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SshRequest {
    pub server_id: i32,
    pub command: String,           // 要执行的命令
}
#[derive(Debug, Deserialize)]
pub struct BatchSshRequest {
    pub group_id: i32,
    pub command: String,           // 要执行的命令
}
#[derive(Debug, Serialize)]
pub struct SshResponse {
    pub exit_code: u32,
    pub output: String,
}


#[derive(Debug, Clone,Serialize,Deserialize)]
pub struct SshError {
    pub server: String,
    pub output: String,
    pub exit_code: Option<i32>
}

impl std::fmt::Display for SshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.server, self.output)
    }
}


#[derive(Debug, Clone,Serialize,Deserialize)]
pub struct SshResult {
    pub server: String,
    pub output: String,
    pub exit_code: Option<u32>,
}