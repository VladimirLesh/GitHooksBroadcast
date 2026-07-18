use crate::error::AppError;
use crate::event::{Actor, CommitInfo, Event, EventKind, PrAction, Repo};
use crate::hmac_verify::verify_hex_sha256;
use crate::sources::SourceKind;
use axum::http::HeaderMap;
use serde_json::Value;

pub fn verify(secret: &[u8], headers: &HeaderMap, body: &[u8]) -> Result<(), AppError> {
    let sig = headers.get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::MissingSignature)?;
    if verify_hex_sha256(secret, body, sig) { Ok(()) } else { Err(AppError::InvalidSignature) }
}

pub fn parse(headers: &HeaderMap, body: &[u8]) -> Result<Event, AppError> {
    let event_type = headers.get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::InvalidPayload("missing X-GitHub-Event".into()))?
        .to_string();

    let v: Value = serde_json::from_slice(body)
        .map_err(|e| AppError::InvalidPayload(format!("json: {e}")))?;

    let repo = Repo {
        full_name: v["repository"]["full_name"].as_str().unwrap_or("").to_string(),
        url: v["repository"]["html_url"].as_str().unwrap_or("").to_string(),
    };
    let actor = Actor {
        login: v["sender"]["login"].as_str().unwrap_or("").to_string(),
        url: v["sender"]["html_url"].as_str().map(String::from),
    };

    let kind = match event_type.as_str() {
        "ping" => return Err(AppError::Ignored),
        "push" => {
            let branch = v["ref"].as_str().unwrap_or("").trim_start_matches("refs/heads/").to_string();
            let commits = v["commits"].as_array().map(|arr| arr.iter().map(|c| CommitInfo {
                id: c["id"].as_str().unwrap_or("").to_string(),
                message: c["message"].as_str().unwrap_or("").to_string(),
                url: c["url"].as_str().unwrap_or("").to_string(),
                author: c["author"]["name"].as_str().unwrap_or("").to_string(),
            }).collect()).unwrap_or_default();
            EventKind::Push {
                branch,
                commits,
                compare_url: v["compare"].as_str().map(String::from),
                forced: v["forced"].as_bool().unwrap_or(false),
            }
        }
        "pull_request" => {
            let action_s = v["action"].as_str().unwrap_or("");
            let pr = &v["pull_request"];
            let merged = pr["merged"].as_bool().unwrap_or(false);
            let action = match action_s {
                "opened" => PrAction::Opened,
                "closed" => PrAction::Closed { merged },
                "reopened" => PrAction::Reopened,
                "synchronize" => PrAction::Synchronized,
                "edited" => PrAction::Edited,
                _ => return Err(AppError::Ignored),
            };
            EventKind::PullRequest {
                action,
                number: v["number"].as_u64().unwrap_or(0),
                title: pr["title"].as_str().unwrap_or("").to_string(),
                url: pr["html_url"].as_str().unwrap_or("").to_string(),
                base: pr["base"]["ref"].as_str().unwrap_or("").to_string(),
                head: pr["head"]["ref"].as_str().unwrap_or("").to_string(),
            }
        }
        _ => return Err(AppError::Ignored),
    };

    Ok(Event { source: SourceKind::Github, repo, actor, kind })
}
