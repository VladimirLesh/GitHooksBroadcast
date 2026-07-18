use crate::sources::SourceKind;

#[derive(Debug, Clone)]
pub struct Event {
    pub source: SourceKind,
    pub repo: Repo,
    pub actor: Actor,
    pub kind: EventKind,
}

#[derive(Debug, Clone)]
pub struct Repo {
    pub full_name: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub login: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EventKind {
    Push {
        branch: String,
        commits: Vec<CommitInfo>,
        compare_url: Option<String>,
        forced: bool,
    },
    PullRequest {
        action: PrAction,
        number: u64,
        title: String,
        url: String,
        base: String,
        head: String,
    },
}

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub url: String,
    pub author: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrAction {
    Opened,
    Closed { merged: bool },
    Reopened,
    Synchronized,
    Edited,
}

impl PrAction {
    pub fn verb(self) -> &'static str {
        match self {
            PrAction::Opened => "opened",
            PrAction::Closed { merged: true } => "merged",
            PrAction::Closed { merged: false } => "closed",
            PrAction::Reopened => "reopened",
            PrAction::Synchronized => "updated",
            PrAction::Edited => "edited",
        }
    }
}
