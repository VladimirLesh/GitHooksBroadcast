use crate::http_client::client;
use crate::render::Rendered;
use anyhow::{anyhow, Result};
use serde_json::json;

pub async fn send(token: &str, chat_id: &str, msg: &Rendered) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{token}/sendMessage");
    let body = json!({
        "chat_id": chat_id,
        "text": msg.html,
        "parse_mode": "HTML",
        "disable_web_page_preview": true,
    });
    let resp = client().post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "telegram {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }
    Ok(())
}
