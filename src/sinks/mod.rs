pub mod discord;
pub mod element;
pub mod mattermost;
pub mod pachca;
pub mod telegram;

use crate::config::SinkCfg;
use crate::render::Rendered;

pub async fn send(id: &str, cfg: &SinkCfg, msg: &Rendered) {
    let result = match cfg {
        SinkCfg::Telegram { token, chat_id } => telegram::send(token, chat_id, msg).await,
        SinkCfg::Discord { webhook_url } => discord::send(webhook_url, msg).await,
        SinkCfg::Element { homeserver, access_token, room_id } => element::send(homeserver, access_token, room_id, msg).await,
        SinkCfg::Mattermost { webhook_url } => mattermost::send(webhook_url, msg).await,
        SinkCfg::Pachca { access_token, entity_id, entity_type } => pachca::send(access_token, *entity_id, entity_type, msg).await,
    };
    if let Err(e) = result {
        tracing::warn!(sink = id, error = %e, "sink send failed");
    } else {
        tracing::debug!(sink = id, "sink send ok");
    }
}
