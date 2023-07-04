use rcon_rs::Client;

use crate::config::Server;

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
