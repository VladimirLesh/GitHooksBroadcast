use crate::event::{Event, EventKind};

pub struct Rendered {
    pub plain: String,
    pub html: String,
    pub markdown: String,
}

const COMMITS_LIMIT: usize = 10;
const SUBJECT_LIMIT: usize = 120;

pub fn render(ev: &Event) -> Rendered {
    match &ev.kind {
        EventKind::Push {
            branch,
            commits,
            compare_url,
            forced,
        } => {
            let n = commits.len();
            let plural = if n == 1 { "commit" } else { "commits" };
            let force = if *forced { " (force-pushed)" } else { "" };
            let repo = &ev.repo.full_name;
            let actor = &ev.actor.login;

            let mut plain = format!("[{repo}] {actor} pushed {n} {plural} to {branch}{force}\n");
            let mut html = format!(
                "<b>[{}]</b> <a href=\"{}\">{}</a> pushed <b>{}</b> {} to <code>{}</code>{}\n",
                esc_html(repo),
                esc_attr(ev.actor.url.as_deref().unwrap_or("")),
                esc_html(actor),
                n,
                plural,
                esc_html(branch),
                force
            );
            let mut md = format!(
                "**\\[{}]** {} pushed **{}** {} to `{}`{}\n",
                md_esc(repo),
                md_esc(actor),
                n,
                plural,
                md_esc(branch),
                force
            );

            for c in commits.iter().take(COMMITS_LIMIT) {
                let short = short_id(&c.id);
                let subject = subject(&c.message);
                plain.push_str(&format!("  {short} {} — {}\n", subject, c.author));
                html.push_str(&format!(
                    "• <a href=\"{}\"><code>{}</code></a> {} — <i>{}</i>\n",
                    esc_attr(&c.url),
                    short,
                    esc_html(&subject),
                    esc_html(&c.author)
                ));
                md.push_str(&format!(
                    "- [`{}`]({}) {} — _{}_\n",
                    short,
                    c.url,
                    md_esc(&subject),
                    md_esc(&c.author)
                ));
            }
            if n > COMMITS_LIMIT {
                let more = n - COMMITS_LIMIT;
                plain.push_str(&format!("  … and {more} more\n"));
                html.push_str(&format!("<i>… and {more} more</i>\n"));
                md.push_str(&format!("_… and {more} more_\n"));
            }
            if let Some(url) = compare_url {
                plain.push_str(&format!("{url}\n"));
                html.push_str(&format!("<a href=\"{}\">compare</a>", esc_attr(url)));
                md.push_str(&format!("[compare]({})", url));
            }
            Rendered {
                plain,
                html,
                markdown: md,
            }
        }
        EventKind::PullRequest {
            action,
            number,
            title,
            url,
            base,
            head,
        } => {
            let repo = &ev.repo.full_name;
            let actor = &ev.actor.login;
            let verb = action.verb();
            let plain =
                format!("[{repo}] {actor} {verb} PR #{number}: {title} ({head} → {base})\n{url}");
            let html  = format!("<b>[{}]</b> <a href=\"{}\">{}</a> <b>{}</b> <a href=\"{}\">PR #{}</a>: {} (<code>{}</code> → <code>{}</code>)",
                                esc_html(repo),
                                esc_attr(ev.actor.url.as_deref().unwrap_or("")),
                                esc_html(actor),
                                verb, esc_attr(url), number, esc_html(title),
                                esc_html(head), esc_html(base));
            let md = format!(
                "**\\[{}]** {} **{}** [PR #{}]({}): {} (`{}` → `{}`)",
                md_esc(repo),
                md_esc(actor),
                verb,
                number,
                url,
                md_esc(title),
                md_esc(head),
                md_esc(base)
            );
            Rendered {
                plain,
                html,
                markdown: md,
            }
        }
    }
}

fn short_id(id: &str) -> &str {
    &id[..id.len().min(7)]
}

fn subject(msg: &str) -> String {
    let first = msg.lines().next().unwrap_or("").trim();
    if first.chars().count() > SUBJECT_LIMIT {
        let mut truncated: String = first.chars().take(SUBJECT_LIMIT).collect();
        truncated.push('…');
        truncated
    } else {
        first.to_string()
    }
}

fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn esc_attr(s: &str) -> String {
    esc_html(s).replace('"', "&quot;")
}

// Markdown escape (works for CommonMark; Mattermost is close enough).
fn md_esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(
            c,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '('
                | ')'
                | '#'
                | '+'
                | '-'
                | '!'
                | '|'
                | '>'
        ) {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
