use crate::http_client::client;
use crate::render::Rendered;
use anyhow::{anyhow, Result};
use serde_json::json;

pub async fn send(webhook_url: &str, msg: &Rendered) -> Result<()> {
    let body = json!({ "text": msg.markdown });
    let resp = client().post(webhook_url).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "mattermost {}: {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }
    Ok(())
}
