use axum::http::HeaderMap;
use gitbroadcast::error::AppError;
use gitbroadcast::event::{EventKind, PrAction};
use gitbroadcast::sources::{self, SourceKind};

fn hdrs(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut h = HeaderMap::new();
    for (k, v) in pairs {
        h.insert(*k, v.parse().unwrap());
    }
    h
}

#[test]
fn parses_github_push() {
    let body = include_bytes!("fixtures/github_push.json");
    let h = hdrs(&[("x-github-event", "push")]);
    let ev = sources::parse(SourceKind::Github, &h, body).unwrap();
    assert_eq!(ev.repo.full_name, "octo/hello");
    match ev.kind {
        EventKind::Push {
            branch,
            commits,
            forced,
            ..
        } => {
            assert_eq!(branch, "main");
            assert_eq!(commits.len(), 2);
            assert!(!forced);
        }
        _ => panic!("expected Push"),
    }
}

#[test]
fn parses_github_pr() {
    let body = include_bytes!("fixtures/github_pr.json");
    let h = hdrs(&[("x-github-event", "pull_request")]);
    let ev = sources::parse(SourceKind::Github, &h, body).unwrap();
    match ev.kind {
        EventKind::PullRequest {
            action,
            number,
            head,
            base,
            ..
        } => {
            assert_eq!(action, PrAction::Opened);
            assert_eq!(number, 42);
            assert_eq!(head, "feature/cool");
            assert_eq!(base, "main");
        }
        _ => panic!("expected PR"),
    }
}

#[test]
fn parses_gitlab_push() {
    let body = include_bytes!("fixtures/gitlab_push.json");
    let h = hdrs(&[("x-gitlab-event", "Push Hook")]);
    let ev = sources::parse(SourceKind::Gitlab, &h, body).unwrap();
    match ev.kind {
        EventKind::Push {
            branch, commits, ..
        } => {
            assert_eq!(branch, "main");
            assert_eq!(commits.len(), 1);
        }
        _ => panic!("expected Push"),
    }
    assert_eq!(ev.repo.full_name, "grp/proj");
    assert_eq!(ev.actor.login, "alice");
}

#[test]
fn parses_gitlab_mr_merge() {
    let body = include_bytes!("fixtures/gitlab_mr.json");
    let h = hdrs(&[("x-gitlab-event", "Merge Request Hook")]);
    let ev = sources::parse(SourceKind::Gitlab, &h, body).unwrap();
    match ev.kind {
        EventKind::PullRequest { action, number, .. } => {
            assert_eq!(action, PrAction::Closed { merged: true });
            assert_eq!(number, 7);
        }
        _ => panic!("expected PR"),
    }
}

#[test]
fn detect_falls_back_on_github() {
    let h = hdrs(&[("x-github-event", "push")]);
    assert_eq!(sources::detect(&h).unwrap(), SourceKind::Github);
}

#[test]
fn detect_prefers_gitea() {
    let h = hdrs(&[("x-github-event", "push"), ("x-gitea-event", "push")]);
    assert_eq!(sources::detect(&h).unwrap(), SourceKind::Gitea);
}

#[test]
fn detect_missing_is_error() {
    let h = HeaderMap::new();
    assert!(matches!(
        sources::detect(&h),
        Err(AppError::UnknownSource(_))
    ));
}

#[test]
fn ignores_github_ping() {
    let body = b"{}";
    let h = hdrs(&[("x-github-event", "ping")]);
    assert!(matches!(
        sources::parse(SourceKind::Github, &h, body),
        Err(AppError::Ignored)
    ));
}
