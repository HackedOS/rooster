use bollard::{container::{ListContainersOptions, AttachContainerOptions}, Docker};
use futures_util::StreamExt;
use rcon_rs::Client;
use regex::Regex;
use serenity::{prelude::Context, model::prelude::ChannelId};

use crate::{config::Server, CONFIG};

impl Server {
    pub async fn send_chat(&self, msg: &str) {
        let lines: Vec<&str> = msg.lines().collect();
        for line in lines {
            if let Some(msg) = clear_formatting(line) {
                let message = format!("tellraw @a {{ \"text\": \"{msg}\" }}");
                let _ = self.rcon_send(&message).await;
            }
        }
    }
    pub async fn rcon_send(&self, msg: &str) {
        let mut conn = self.connect().await;
        if conn.auth(&self.password).is_ok() {
            let _ = conn.send(dbg!(msg), None);
        };
    }
    async fn connect(&self) -> Client {
        Client::new(&self.ip, &self.port.to_string())
    }
}

#[inline(always)]
fn clear_formatting(msg: &str) -> Option<String> {
    let msg = msg
        .replace('\\', "")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('{', "{{")
        .replace('}', "}}")
        .replace('\"', "\\\"");
    match msg.len() {
        1.. => Some(msg),
        _ => None,
    }
}

async fn check_container_status(docker: &Docker, container_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let options = ListContainersOptions::<String> {
        all: true,
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;
    
    for container in containers {
        if let Some(names) = container.names {
            for name in names {
                if name == format!("/{}", container_name) {
                    return Ok(container.state == Some("running".to_string()));
                }
            }
        }
    }

    Ok(false)
}

pub async fn chatbridge(docker: &Docker, server: Server, ctx: Context) {
    let mut output = docker
                .attach_container(
                    &server.container_name,
                    Some(AttachContainerOptions::<String> {
                        stdout: Some(true),
                        stderr: Some(true),
                        stream: Some(true),
                        ..Default::default()
                    }),
                )
                .await
                .unwrap()
                .output;
            tokio::spawn(async move {
                while let Some(Ok(output)) = output.next().await {
                    let parse_pattern = Regex::new(r"^\[\d{2}:\d{2}:\d{2}\] \[Server thread/INFO\]: (<.*|[\w §]+ (joined|left) the game)$").unwrap();
                    if !parse_pattern.is_match(output.to_string().trim()) {
                        continue;
                    }
                    let msg = &format!(
                        "[{}]: {}",
                        server.display_name,
                        output.to_string().chars().collect::<Vec<char>>()[33..]
                            .iter()
                            .collect::<String>()
                    );
                    ChannelId(CONFIG.bridge_channel)
                        .send_message(&ctx.http, |m| {
                            m.content(msg)
                        })
                        .await
                        .unwrap();
                    let mut send_servers = CONFIG.servers.clone();
                    send_servers.retain(|s| s != &server);
                    for server in send_servers {
                        server.send_chat(msg).await
                    }
                }
            });
}