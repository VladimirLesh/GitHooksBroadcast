use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    pub server: ServerCfg,
    #[serde(default)]
    pub routes: Vec<RouteCfg>,
    #[serde(default)]
    pub sinks: BTreeMap<String, SinkCfg>,
}

#[derive(Debug, Deserialize)]
pub struct ServerCfg {
    pub bind: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteCfg {
    pub id: String,
    pub secret: String,
    #[serde(default)]
    pub sinks: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SinkCfg {
    Telegram { token: String, chat_id: String },
    Discord { webhook_url: String },
    Element { homeserver: String, access_token: String, room_id: String },
    Mattermost { webhook_url: String },
    Pachca { access_token: String, entity_id: u64, entity_type: String },
}

pub struct Config {
    pub bind: SocketAddr,
    pub routes: BTreeMap<String, LoadedRoute>,
    pub sinks: BTreeMap<String, SinkCfg>,
}

pub struct LoadedRoute {
    pub secret: Vec<u8>,
    pub sinks: Vec<String>,
}

pub fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let raw: RawConfig = toml::from_str(&text).context("parsing TOML")?;

    let bind: SocketAddr = raw.server.bind.parse()
        .with_context(|| format!("parsing server.bind = {:?}", raw.server.bind))?;

    let mut sinks = BTreeMap::new();
    for (id, s) in raw.sinks {
        sinks.insert(id, resolve_sink(s)?);
    }

    let mut routes = BTreeMap::new();
    for r in raw.routes {
        for sink_id in &r.sinks {
            if !sinks.contains_key(sink_id) {
                return Err(anyhow!("route {}: unknown sink {}", r.id, sink_id));
            }
        }
        let secret = resolve_env(&r.secret).context("route secret")?.into_bytes();
        if secret.is_empty() {
            return Err(anyhow!("route {}: empty secret", r.id));
        }
        routes.insert(r.id.clone(), LoadedRoute { secret, sinks: r.sinks });
    }

    Ok(Config { bind, routes, sinks })
}

fn resolve_sink(s: SinkCfg) -> Result<SinkCfg> {
    Ok(match s {
        SinkCfg::Telegram { token, chat_id } => SinkCfg::Telegram {
            token: resolve_env(&token)?,
            chat_id: resolve_env(&chat_id)?,
        },
        SinkCfg::Discord { webhook_url } => SinkCfg::Discord {
            webhook_url: resolve_env(&webhook_url)?,
        },
        SinkCfg::Element { homeserver, access_token, room_id } => SinkCfg::Element {
            homeserver: resolve_env(&homeserver)?,
            access_token: resolve_env(&access_token)?,
            room_id: resolve_env(&room_id)?,
        },
        SinkCfg::Mattermost { webhook_url } => SinkCfg::Mattermost {
            webhook_url: resolve_env(&webhook_url)?,
        },
        SinkCfg::Pachca { access_token, entity_id, entity_type } => SinkCfg::Pachca {
            access_token: resolve_env(&access_token)?,
            entity_id,
            entity_type: resolve_env(&entity_type)?,
        },
    })
}

fn resolve_env(v: &str) -> Result<String> {
    if let Some(name) = v.strip_prefix("$env:") {
        std::env::var(name).map_err(|_| anyhow!("environment variable {} not set", name))
    } else {
        Ok(v.to_string())
    }
}
