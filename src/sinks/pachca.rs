use crate::http_client::client;
use crate::render::Rendered;
use anyhow::{anyhow, Result};
use serde_json::json;

pub async fn send(token: &str, entity_id: u64, entity_type: &str, msg: &Rendered) -> Result<()> {
    let url = "https://api.pachca.com/api/shared/v1/messages";
    let body = json!({
        "message": {
            "entity_type": entity_type,
            "entity_id": entity_id,
            "content": msg.markdown,
        }
    });
    let resp = client().post(url).bearer_auth(token).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow!("pachca {}: {}", resp.status(), resp.text().await.unwrap_or_default()));
    }
    Ok(())
}
