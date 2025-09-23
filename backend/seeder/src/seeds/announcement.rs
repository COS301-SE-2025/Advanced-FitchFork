use crate::seed::Seeder;
use chrono::{Duration, Utc};
use db::models::{
    announcements::{ActiveModel as AnnouncementActiveModel, Entity as AnnouncementEntity},
    module, user,
    user_module_role::{self, Role as ModuleRole},
};
use rand::rngs::{OsRng, StdRng};
use rand::{Rng, SeedableRng, seq::SliceRandom};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct AnnouncementSeeder;

impl Seeder for AnnouncementSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            let mut rng = StdRng::from_rng(OsRng).expect("rng");

            let all_users = UserService::find_all(&vec![], &vec![], None).await?;
            if all_users.is_empty() {
                panic!("No users found; run UserSeeder first");
            }

            let modules = ModuleService::find_all(&vec![], &vec![], None).await?;
            if modules.is_empty() {
                return Err(AppError::DatabaseUnknown);
            }

            let titles = [
                "Important update",
                "Reminder",
                "Schedule notice",
                "New resource available",
                "General announcement",
                "Assessment info",
                "Administrative note",
                "Office hour change",
                "Lab session update",
                "Reading list update",
            ];

            for m in modules {
                // Prefer module staff as authors
                let staff_user_ids: Vec<i64> = user_module_role::Entity::find()
                    .filter(user_module_role::Column::ModuleId.eq(m.id))
                    .filter(user_module_role::Column::Role.is_in(vec![
                        ModuleRole::Lecturer,
                        ModuleRole::AssistantLecturer,
                        ModuleRole::Tutor,
                    ]))
                    .all(db)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|r| r.user_id)
                    .collect();

                let pick_author = |rng: &mut StdRng| -> i64 {
                    if !staff_user_ids.is_empty() {
                        *staff_user_ids.choose(rng).unwrap()
                    } else {
                        all_users.choose(rng).unwrap().id
                    }
                };

                for i in 0..20 {
                    let title = titles.choose(&mut rng).unwrap().to_string();

                    // Spread creation dates across the last ~6 months
                    let created_at = Utc::now()
                        - Duration::days(rng.gen_range(0..=180))
                        - Duration::hours(rng.gen_range(0..=23))
                        - Duration::minutes(rng.gen_range(0..=59));

                    let body = build_long_markdown(&m.code, m.year, created_at, &mut rng);

                    // Ensure a couple are pinned per module; others ~22% chance
                    let pinned = if i < 3 { true } else { rng.gen_bool(0.22) };

                    AnnouncementService::create(CreateAnnouncement {
                        module_id: m.id,
                        user_id: pick_author(&mut rng),
                        title: title,
                        body: body,
                        pinned: pinned,
                    })
                    .await?;
                }
            }

            Ok(())
        })
    }
}

/// Build a rich, longer Markdown body with sections, lists, code, quotes, and a simple table.
fn build_long_markdown(
    code: &str,
    year: i32,
    ts: chrono::DateTime<Utc>,
    rng: &mut StdRng,
) -> String {
    let when = ts.format("%Y-%m-%d %H:%M").to_string();

    let intros = [
        format!(
            "## Context\nWe’re sharing a detailed update for **{code} {year}**. Please read carefully and plan accordingly. \
            This note consolidates recent questions, logistics, and recommended next steps."
        ),
        format!(
            "## Heads-up\nThis announcement for **{code} {year}** covers scheduling, resources, and important reminders. \
            If you’re short on time, skim the lists below and revisit the details later."
        ),
        format!(
            "## Overview\nBelow you’ll find the latest information related to **{code} {year}**. \
            We’ve included timelines, links, and a short FAQ to make it easy to follow."
        ),
    ];

    let bullets_pool = [
        "Review the updated outline and confirm your understanding.",
        "Check the LMS for supplementary notes and examples.",
        "Form or confirm study groups and share availability.",
        "Complete the practice set before the next tutorial.",
        "Skim the reference documentation prior to coding.",
        "Push small, frequent commits to keep history readable.",
        "Run the test suite locally before submitting work.",
        "Use meaningful commit messages and avoid force pushes.",
        "Bring any blockers to office hours or the forum.",
        "Keep an eye on deadline windows and grace periods.",
    ];

    let numbered_pool = [
        "Read the problem statement twice; annotate constraints.",
        "Sketch a small example; identify edge cases.",
        "Draft a minimal prototype; verify basic behavior.",
        "Refactor iteratively; remove duplication.",
        "Add tests for off-by-one and error paths.",
        "Profile if needed; optimize only true hotspots.",
    ];

    let quotes = [
        "> “Make it work, make it right, make it fast.” — Kent Beck",
        "> “Programs must be written for people to read, and only incidentally for machines to execute.” — Harold Abelson",
        "> “Simplicity is prerequisite for reliability.” — Edsger Dijkstra",
    ];

    let links = [
        "[Course homepage](https://example.edu/course)",
        "[Department site](https://example.edu/cs)",
        "[Reference docs](https://example.edu/docs)",
    ];

    let code_blocks = [
        (
            "bash",
            r#"# Run tests locally
cargo test -- --nocapture

# Format and lint
cargo fmt
cargo clippy -- -D warnings"#,
        ),
        (
            "rust",
            r#"fn main() {
    println!("Hello, COS!");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}"#,
        ),
        (
            "json",
            r#"{
  "name": "example",
  "version": "1.0.0",
  "private": true
}"#,
        ),
    ];

    // Pick random slices
    let intro = intros.choose(rng).unwrap().to_string();

    let bullets_count = rng.gen_range(4..=7);
    let bullets = bullets_pool
        .choose_multiple(rng, bullets_count)
        .map(|s| format!("- {}", s))
        .collect::<Vec<_>>()
        .join("\n");

    let numbered_count = rng.gen_range(3..=6);
    let numbered = numbered_pool
        .choose_multiple(rng, numbered_count)
        .enumerate()
        .map(|(i, s)| format!("{}. {}", i + 1, s))
        .collect::<Vec<_>>()
        .join("\n");

    let maybe_quote = if rng.gen_bool(0.7) {
        quotes.choose(rng).map(|q| q.to_string()).unwrap()
    } else {
        String::new()
    };

    let maybe_links = if rng.gen_bool(0.8) {
        let n = rng.gen_range(1..=links.len());
        links
            .choose_multiple(rng, n)
            .map(|l| format!("- {}", l))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

    let maybe_code = if rng.gen_bool(0.65) {
        let (lang, block) = code_blocks.choose(rng).unwrap();
        format!("```{}\n{}\n```", lang, block)
    } else {
        String::new()
    };

    let maybe_table = if rng.gen_bool(0.5) {
        // Simple 3x3 table
        r#"| Section | Item | Status |
|--------:|:-----|:------:|
| Intro   | Notes | ✅     |
| Tasks   | Set A | ⏳     |
| Review  | Q&A   | ✅     |"#
            .to_string()
    } else {
        String::new()
    };

    format!(
        r#"{intro}

### Action Items
{bullets}

### Recommended Approach
{numbered}

{maybe_quote}

{maybe_table}

### Useful Links
{maybe_links}

{maybe_code}

_Updated at {when}_ in **{code} {year}**."#,
        intro = intro,
        bullets = bullets,
        numbered = numbered,
        maybe_quote = maybe_quote,
        maybe_table = maybe_table,
        maybe_links = if maybe_links.is_empty() {
            "- (none)".to_string()
        } else {
            maybe_links
        },
        maybe_code = maybe_code,
        when = when,
        code = code,
        year = year
    )
}
