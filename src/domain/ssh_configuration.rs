use russh::client::Handler;
use std::sync::Arc;
use russh::{client, ChannelMsg, Disconnect};
use russh::keys::*;
use tokio::io::AsyncWriteExt;
use anyhow::Result;
use russh::client::Config;


pub struct Message{
    pub user: String,
    pub password: String,
    pub ipaddr: Option<String>,
    pub port: String,
    pub config: Arc<Config>,
    pub server_list: Option<Vec<String>>
}
impl Message{
    pub fn new(user: String,password: String,port: String,ipaddr: Option<String>,server_list: Option<Vec<String>>) -> Self {
        let config = Arc::new(russh::client::Config::default());

        Self{
            user,
            password,
            ipaddr,
            port,
            config,
            server_list,
        }
    }
}


pub struct Client;
impl Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}


pub struct Session {
    pub session: client::Handle<Client>,
}
impl Session {
    pub async fn call(&mut self, command: &str) -> anyhow::Result<(u32,String)> {
        let mut channel = self.session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut code = None;
        let mut stdout = tokio::io::stdout();
        let mut output = Vec::new();
        loop {
            // There's an event available on the session channel
            let Some(msg) = channel.wait().await else {
                break;
            };
             match msg {
                // Write data to the terminal
                ChannelMsg::Data { ref data } => {
                    output.extend_from_slice(data);
                    stdout.write_all(data).await?;
                    stdout.flush().await?;

                }
                // The command has returned an exit code
                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                    // cannot leave the loop immediately, there might still be more data to receive
                }
                _ => {}
            }
        }
        let code = code.expect("program did not exit cleanly");
        let output_str = String::from_utf8_lossy(&output).to_string();
        Ok((code,output_str))
    }

    pub async fn close(&mut self) -> anyhow::Result<()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await?;
        Ok(())
    }
}