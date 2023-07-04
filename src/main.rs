mod config;
mod server;

use bollard::container::AttachContainerOptions;
use bollard::Docker;

use config::{load_config, Config};
use futures_util::StreamExt;

use regex::Regex;
use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::StandardFramework;
use serenity::model::prelude::{ChannelId, Message, Ready};
use serenity::prelude::*;

#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        let docker = Docker::connect_with_socket_defaults().unwrap();

        for server in &CONFIG.servers {
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
            let ctx1 = ctx.clone();
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
                        .send_message(&ctx1.http, |m| {
                            m.content(msg)
                        })
                        .await
                        .unwrap();
                    let mut send_servers = CONFIG.servers.clone();
                    send_servers.retain(|s| s != server);
                    for server in send_servers {
                        server.send_chat(msg).await
                    }
                }
            });
        }
    }
    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.channel_id.0 != CONFIG.bridge_channel || msg.author.bot {
            return;
        }
            for server in &CONFIG.servers {
                server.send_chat(&format!("[{0}] {1}", msg.author.name, msg.content)).await
            }
        }
}

lazy_static::lazy_static! {
pub static ref CONFIG: Config = load_config();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // set the bot's prefix to "!"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&CONFIG.discord_token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}
