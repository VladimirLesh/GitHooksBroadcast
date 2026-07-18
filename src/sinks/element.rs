use crate::http_client::client;
use crate::render::Rendered;
use anyhow::{anyhow, Result};
use serde_json::json;

pub async fn send(homeserver: &str, access_token: &str, room_id: &str, msg: &Rendered) -> Result<()> {
    let homeserver = homeserver.trim_end_matches('/');
    let encoded_room = urlencoding_encode(room_id);
    let txn = uuid::Uuid::new_v4();
    let url = format!("{homeserver}/_matrix/client/v3/rooms/{encoded_room}/send/m.room.message/{txn}");
    let body = json!({
        "msgtype": "m.text",
        "body": msg.plain,
        "format": "org.matrix.custom.html",
        "formatted_body": msg.html,
    });
    let resp = client().put(&url).bearer_auth(access_token).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("matrix {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }
    Ok(())
}

// Minimal percent-encoder for path segments (Matrix room IDs contain '!' and ':').
fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        let safe = matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~');
        if safe { out.push(b as char); } else { out.push_str(&format!("%{b:02X}")); }
    }
    out
}
