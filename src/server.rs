use crate::config::{Config, SinkCfg};
use crate::error::AppError;
use crate::{render, sinks, sources};
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use bytes::Bytes;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

pub struct AppState {
    pub bind_addr: SocketAddr,
    pub routes: BTreeMap<String, Arc<RouteState>>,
    pub sinks: Arc<BTreeMap<String, SinkCfg>>,
}

pub struct RouteState {
    pub secret: Vec<u8>,
    pub sink_ids: Vec<String>,
}

impl AppState {
    pub fn from_config(cfg: Config) -> anyhow::Result<Self> {
        let sinks = Arc::new(cfg.sinks);
        let routes = cfg
            .routes
            .into_iter()
            .map(|(id, r)| {
                (
                    id,
                    Arc::new(RouteState {
                        secret: r.secret,
                        sink_ids: r.sinks,
                    }),
                )
            })
            .collect();
        Ok(Self {
            bind_addr: cfg.bind,
            routes,
            sinks,
        })
    }
}

#[derive(Clone)]
pub struct SharedState(pub Arc<AppState>);

pub fn router(state: AppState) -> Router {
    let shared = SharedState(Arc::new(state));
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/hook/:route_id", post(handle_hook))
        .with_state(shared)
}

async fn handle_hook(
    State(state): State<SharedState>,
    Path(route_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let route = state
        .0
        .routes
        .get(&route_id)
        .ok_or(AppError::RouteNotFound)?
        .clone();

    let source = sources::detect(&headers)?;
    sources::verify(source, &route.secret, &headers, &body)?;
    let event = sources::parse(source, &headers, &body)?;

    tracing::info!(route = %route_id, source = source.as_str(), repo = %event.repo.full_name, "event accepted");

    let rendered = render::render(&event);
    let sinks_map = state.0.sinks.clone();
    let sink_ids = route.sink_ids.clone();

    tokio::spawn(async move {
        for id in sink_ids {
            if let Some(cfg) = sinks_map.get(&id) {
                sinks::send(&id, cfg, &rendered).await;
            }
        }
    });

    Ok(StatusCode::ACCEPTED)
}
