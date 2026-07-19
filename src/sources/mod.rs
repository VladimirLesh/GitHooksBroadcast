pub mod gitea;
pub mod github;
pub mod gitlab;

use crate::error::AppError;
use crate::event::Event;
use axum::http::HeaderMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Github,
    Gitlab,
    Gitea,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Github => "github",
            SourceKind::Gitlab => "gitlab",
            SourceKind::Gitea => "gitea",
        }
    }
}

pub fn detect(headers: &HeaderMap) -> Result<SourceKind, AppError> {
    // Order matters: Gitea sets X-Gitea-Event AND sometimes X-GitHub-Event for compat,
    // so check Gitea first.
    if headers.contains_key("x-gitea-event") {
        return Ok(SourceKind::Gitea);
    }
    if headers.contains_key("x-gitlab-event") {
        return Ok(SourceKind::Gitlab);
    }
    if headers.contains_key("x-github-event") {
        return Ok(SourceKind::Github);
    }
    Err(AppError::UnknownSource("no known X-*-Event header".into()))
}

pub fn verify(
    kind: SourceKind,
    secret: &[u8],
    headers: &HeaderMap,
    body: &[u8],
) -> Result<(), AppError> {
    match kind {
        SourceKind::Github => github::verify(secret, headers, body),
        SourceKind::Gitlab => gitlab::verify(secret, headers),
        SourceKind::Gitea => gitea::verify(secret, headers, body),
    }
}

pub fn parse(kind: SourceKind, headers: &HeaderMap, body: &[u8]) -> Result<Event, AppError> {
    match kind {
        SourceKind::Github => github::parse(headers, body),
        SourceKind::Gitlab => gitlab::parse(headers, body),
        SourceKind::Gitea => gitea::parse(headers, body),
    }
}
