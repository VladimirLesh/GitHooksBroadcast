use crate::error::AppError;
use crate::event::{Actor, CommitInfo, Event, EventKind, PrAction, Repo};
use crate::hmac_verify::verify_shared_secret;
use crate::sources::SourceKind;
use axum::http::HeaderMap;
use serde_json::Value;

// GitLab uses a plain shared-secret token (not HMAC) in X-Gitlab-Token.
pub fn verify(secret: &[u8], headers: &HeaderMap) -> Result<(), AppError> {
    let token = headers
        .get("x-gitlab-token")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::MissingSignature)?;
    if verify_shared_secret(secret, token.as_bytes()) {
        Ok(())
    } else {
        Err(AppError::InvalidSignature)
    }
}

pub fn parse(headers: &HeaderMap, body: &[u8]) -> Result<Event, AppError> {
    let event_type = headers
        .get("x-gitlab-event")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::InvalidPayload("missing X-Gitlab-Event".into()))?
        .to_string();

    let v: Value =
        serde_json::from_slice(body).map_err(|e| AppError::InvalidPayload(format!("json: {e}")))?;

    // GitLab payloads use different top-level shapes for push vs. MR.
    let project = &v["project"];
    let repo = Repo {
        full_name: project["path_with_namespace"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        url: project["web_url"].as_str().unwrap_or("").to_string(),
    };
    let user_login = v["user_username"]
        .as_str()
        .or_else(|| v["user"]["username"].as_str())
        .unwrap_or("");
    let actor = Actor {
        login: user_login.to_string(),
        url: None,
    };

    let kind = match event_type.as_str() {
        "Push Hook" | "Tag Push Hook" => {
            let branch = v["ref"]
                .as_str()
                .unwrap_or("")
                .trim_start_matches("refs/heads/")
                .to_string();
            let commits = v["commits"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|c| CommitInfo {
                            id: c["id"].as_str().unwrap_or("").to_string(),
                            message: c["message"].as_str().unwrap_or("").to_string(),
                            url: c["url"].as_str().unwrap_or("").to_string(),
                            author: c["author"]["name"].as_str().unwrap_or("").to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default();
            EventKind::Push {
                branch,
                commits,
                compare_url: None,
                forced: false,
            }
        }
        "Merge Request Hook" => {
            let attrs = &v["object_attributes"];
            let action_s = attrs["action"].as_str().unwrap_or("");
            let state = attrs["state"].as_str().unwrap_or("");
            let action = match action_s {
                "open" => PrAction::Opened,
                "reopen" => PrAction::Reopened,
                "update" => PrAction::Synchronized,
                "close" => PrAction::Closed { merged: false },
                "merge" => PrAction::Closed { merged: true },
                _ => match state {
                    "opened" => PrAction::Opened,
                    "closed" => PrAction::Closed { merged: false },
                    "merged" => PrAction::Closed { merged: true },
                    _ => return Err(AppError::Ignored),
                },
            };
            EventKind::PullRequest {
                action,
                number: attrs["iid"].as_u64().unwrap_or(0),
                title: attrs["title"].as_str().unwrap_or("").to_string(),
                url: attrs["url"].as_str().unwrap_or("").to_string(),
                base: attrs["target_branch"].as_str().unwrap_or("").to_string(),
                head: attrs["source_branch"].as_str().unwrap_or("").to_string(),
            }
        }
        _ => return Err(AppError::Ignored),
    };

    Ok(Event {
        source: SourceKind::Gitlab,
        repo,
        actor,
        kind,
    })
}
