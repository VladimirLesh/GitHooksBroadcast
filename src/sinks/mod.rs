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
        SinkCfg::Element {
            homeserver,
            access_token,
            room_id,
        } => element::send(homeserver, access_token, room_id, msg).await,
        SinkCfg::Mattermost { webhook_url } => mattermost::send(webhook_url, msg).await,
        SinkCfg::Pachca {
            access_token,
            entity_id,
            entity_type,
        } => pachca::send(access_token, *entity_id, entity_type, msg).await,
    };
    if let Err(e) = result {
        let msg = mask_secrets(&format!("{e:#}"));
        tracing::warn!(sink = id, error = %msg, "sink send failed");
    } else {
        tracing::debug!(sink = id, "sink send ok");
    }
}

// Strip secrets from error messages before logging. reqwest's error Display
// includes the full request URL, which for Telegram Bot API contains the token.
fn mask_secrets(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find("/bot") {
        out.push_str(&rest[..pos + 4]);
        rest = &rest[pos + 4..];
        // Consume the token: digits + ':' + [A-Za-z0-9_-]+
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == ':' || c == '_' || c == '-'))
            .unwrap_or(rest.len());
        if end > 0 {
            out.push_str("[REDACTED]");
            rest = &rest[end..];
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::mask_secrets;

    #[test]
    fn masks_telegram_bot_token() {
        let s = "error sending request for url (https://api.telegram.org/bot123456:ABC-def_ghi/sendMessage)";
        assert_eq!(
            mask_secrets(s),
            "error sending request for url (https://api.telegram.org/bot[REDACTED]/sendMessage)"
        );
    }

    #[test]
    fn passes_through_non_matching() {
        let s = "matrix 404 Not Found";
        assert_eq!(mask_secrets(s), s);
    }
}
