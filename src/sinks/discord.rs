use crate::http_client::client;
use crate::render::Rendered;
use anyhow::{anyhow, Result};
use serde_json::json;

pub async fn send(webhook_url: &str, msg: &Rendered) -> Result<()> {
    // Discord accepts plain markdown in `content`; cap at 2000 chars per API.
    let mut content = msg.markdown.clone();
    if content.chars().count() > 1990 {
        content = content.chars().take(1990).collect::<String>() + "…";
    }
    let body = json!({ "content": content, "allowed_mentions": { "parse": [] } });
    let resp = client().post(webhook_url).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "discord {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }
    Ok(())
}
