use gitbroadcast::event::{Actor, CommitInfo, Event, EventKind, PrAction, Repo};
use gitbroadcast::render;
use gitbroadcast::sources::SourceKind;

fn push_event() -> Event {
    Event {
        source: SourceKind::Github,
        repo: Repo {
            full_name: "octo/hello".into(),
            url: "https://gh/octo/hello".into(),
        },
        actor: Actor {
            login: "octocat".into(),
            url: Some("https://gh/octocat".into()),
        },
        kind: EventKind::Push {
            branch: "main".into(),
            commits: vec![CommitInfo {
                id: "abcdef1234".into(),
                message: "fix: <script>alert(1)</script>".into(),
                url: "https://gh/octo/hello/commit/abcdef1".into(),
                author: "Alice".into(),
            }],
            compare_url: Some("https://gh/octo/hello/compare/x...y".into()),
            forced: true,
        },
    }
}

#[test]
fn html_is_escaped() {
    let r = render::render(&push_event());
    assert!(r.html.contains("&lt;script&gt;"));
    assert!(!r.html.contains("<script>"));
}

#[test]
fn plain_and_markdown_present() {
    let r = render::render(&push_event());
    assert!(r.plain.contains("octocat pushed 1 commit"));
    assert!(r.plain.contains("(force-pushed)"));
    assert!(r.markdown.contains("`main`"));
    assert!(r.markdown.contains("abcdef1"));
}

#[test]
fn pr_renders_verb() {
    let ev = Event {
        source: SourceKind::Github,
        repo: Repo {
            full_name: "o/r".into(),
            url: "u".into(),
        },
        actor: Actor {
            login: "alice".into(),
            url: None,
        },
        kind: EventKind::PullRequest {
            action: PrAction::Closed { merged: true },
            number: 12,
            title: "T".into(),
            url: "https://gh/o/r/pull/12".into(),
            base: "main".into(),
            head: "topic".into(),
        },
    };
    let r = render::render(&ev);
    assert!(r.plain.contains("merged PR #12"));
    assert!(r.html.contains("merged"));
}
