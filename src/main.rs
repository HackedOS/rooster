mod config;
mod server;

use config::{load_config, Config};
use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::StandardFramework;
use serenity::model::prelude::{Message, Ready};
use serenity::prelude::*;
use server::chatbridge_keepalive;

#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        for server in &CONFIG.servers {
            chatbridge_keepalive(server.clone(), ctx.clone());
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
