use std::path::Path;
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use regex::Regex;
use reqwest::{redirect, Client};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fs::{create_dir_all, File};

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Full,
    Minimal,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// The MOSS result URL (serves HTML)
    url: String,
    /// Print raw HTML (debug)
    #[arg(long)]
    print_html: bool,
    /// Keep only matches with at least this many lines
    #[arg(long, default_value_t = 0)]
    min_lines: i64,
    /// Output JSON path (relative to project). Defaults to "moss_report.json"
    #[arg(long, default_value = "moss_report.json")]
    out: String,
    /// Output format: full (with matches) or minimal (no matches array)
    #[arg(long, value_enum, default_value_t = OutputFormat::Full)]
    format: OutputFormat,
}

#[derive(Debug, serde::Serialize, Clone)]
struct PairRef {
    raw: String,
    username: Option<String>,
    submission_id: Option<i64>,
    filename: Option<String>,
    percent: Option<u32>,
    href: String,
}

#[derive(Debug, serde::Serialize, Clone)]
struct MossPair {
    file1: PairRef,
    file2: PairRef,
    lines_matched: i64,
    match_href: String,
}

#[derive(Debug, serde::Serialize)]
struct FileMatchRow {
    a_filename: String,
    b_filename: String,
    percent: Option<u32>,
    lines_matched: i64,
    match_href: String,
}

#[derive(Debug, serde::Serialize)]
struct UserPairReport {
    user_a: String,
    user_b: String,
    submission_id_a: Option<i64>,
    submission_id_b: Option<i64>,
    total_lines_matched: i64,
    total_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    matches: Option<Vec<FileMatchRow>>,
}

#[derive(Debug, serde::Serialize)]
struct Output {
    title: Option<String>,
    reports: Vec<UserPairReport>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let html = fetch_html(&args.url).await?;
    if args.print_html {
        println!("{html}");
    }

    let doc = Html::parse_document(&html);
    let mut pairs = extract_pairs(&doc);

    pairs.retain(|p| p.file1.username.as_deref() != p.file2.username.as_deref());

    if args.min_lines > 0 {
        pairs.retain(|p| p.lines_matched >= args.min_lines);
    }

    let pairs = dedupe_pairs_keep_best(pairs);
    let reports = group_by_user_pair(pairs, matches_included(&args.format));
    let out = Output {
        title: extract_title(&doc),
        reports,
    };

    println!("{}", serde_json::to_string_pretty(&out)?);

    save_json(&out, &args.out)?;

    eprintln!("Saved report to {}", args.out);
    Ok(())
}

fn matches_included(fmt: &OutputFormat) -> bool {
    matches!(fmt, OutputFormat::Full)
}

fn save_json(out: &Output, path_str: &str) -> Result<()> {
    let path = Path::new(path_str);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            create_dir_all(parent).with_context(|| format!("creating dir {}", parent.display()))?;
        }
    }
    let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
    serde_json::to_writer_pretty(file, out).context("writing JSON")
}

async fn fetch_html(url: &str) -> Result<String> {
    let client = Client::builder()
        .user_agent(concat!(
            "moss-scrape/0.1 (+https://example.invalid) ",
            "reqwest/"
        ))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .http1_only()
        .redirect(redirect::Policy::limited(10))
        .build()
        .context("building HTTP client")?;

    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .context("non-success status")?;

    let bytes = resp.bytes().await.context("reading body")?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn extract_title(doc: &Html) -> Option<String> {
    let sel = Selector::parse("title").unwrap();
    doc.select(&sel)
        .next()
        .map(|t| t.text().collect::<String>().trim().to_string())
}

fn extract_pairs(doc: &Html) -> Vec<MossPair> {
    let tr_sel = Selector::parse("table tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let pct_re = Regex::new(r"^(?P<name>.+?)\s*\((?P<pct>\d+)%\)\s*$").unwrap();

    let mut out = Vec::new();
    for (row_idx, tr) in doc.select(&tr_sel).enumerate() {
        if row_idx == 0 && tr.select(&Selector::parse("th").unwrap()).next().is_some() {
            continue;
        }

        let mut tds = tr.select(&td_sel);
        let (Some(td1), Some(td2), Some(td3)) = (tds.next(), tds.next(), tds.next()) else { continue };

        let a1 = match td1.select(&a_sel).next() { Some(a) => a, None => continue };
        let href1 = a1.value().attr("href").unwrap_or("").to_string();
        let text1 = a1.text().collect::<String>().trim().to_string();
        let (name1, pct1) = parse_name_and_pct(&text1, &pct_re);
        let (u1, s1, f1) = parse_triplet(&name1);

        let a2 = match td2.select(&a_sel).next() { Some(a) => a, None => continue };
        let href2 = a2.value().attr("href").unwrap_or("").to_string();
        let text2 = a2.text().collect::<String>().trim().to_string();
        let (name2, pct2) = parse_name_and_pct(&text2, &pct_re);
        let (u2, s2, f2) = parse_triplet(&name2);

        let lines_txt = td3.text().collect::<String>().trim().to_string();
        let lines_matched = lines_txt.parse::<i64>().unwrap_or(0);

        out.push(MossPair {
            file1: PairRef { raw: name1, username: u1, submission_id: s1, filename: f1, percent: pct1, href: href1.clone() },
            file2: PairRef { raw: name2, username: u2, submission_id: s2, filename: f2, percent: pct2, href: href2.clone() },
            lines_matched,
            match_href: href1,
        });
    }
    out
}

fn parse_triplet(s: &str) -> (Option<String>, Option<i64>, Option<String>) {
    let re = Regex::new(r"^(?P<user>.+)_(?P<id>\d+)/(?P<file>.+)$").unwrap();
    if let Some(c) = re.captures(s) {
        let user = c.name("user").map(|m| m.as_str().to_string());
        let id = c.name("id").and_then(|m| m.as_str().parse::<i64>().ok());
        let file = c.name("file").map(|m| m.as_str().to_string());
        (user, id, file)
    } else {
        (None, None, None)
    }
}

fn parse_name_and_pct(s: &str, re: &Regex) -> (String, Option<u32>) {
    if let Some(c) = re.captures(s) {
        let name = c
            .name("name")
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| s.to_string());
        let pct = c.name("pct").and_then(|m| m.as_str().parse::<u32>().ok());
        (name, pct)
    } else {
        (s.to_string(), None)
    }
}

fn file_identity(f: &PairRef) -> String {
    match (&f.username, &f.submission_id, &f.filename) {
        (Some(u), Some(id), Some(fn_)) => format!("{}|{}|{}", u, id, fn_),
        _ => f.raw.clone(),
    }
}
fn canonical_file_key(a: &PairRef, b: &PairRef) -> (String, String) {
    let ia = file_identity(a);
    let ib = file_identity(b);
    if ia <= ib { (ia, ib) } else { (ib, ia) }
}
fn dedupe_pairs_keep_best(pairs: Vec<MossPair>) -> Vec<MossPair> {
    let mut best: HashMap<(String, String), MossPair> = HashMap::new();
    for p in pairs {
        let key = canonical_file_key(&p.file1, &p.file2);
        match best.get_mut(&key) {
            None => { best.insert(key, p); }
            Some(existing) => {
                if p.lines_matched > existing.lines_matched
                    || (p.lines_matched == existing.lines_matched && p.match_href < existing.match_href)
                {
                    *existing = p;
                }
            }
        }
    }
    let mut out: Vec<MossPair> = best.into_values().collect();
    out.sort_by(|a, b| {
        let (a1, a2) = canonical_file_key(&a.file1, &a.file2);
        let (b1, b2) = canonical_file_key(&b.file1, &b.file2);
        a1.cmp(&b1)
            .then(a2.cmp(&b2))
            .then(b.lines_matched.cmp(&a.lines_matched))
            .then(a.match_href.cmp(&b.match_href))
    });
    out
}

fn group_by_user_pair(pairs: Vec<MossPair>, include_matches: bool) -> Vec<UserPairReport> {
    let mut by_users: HashMap<(String, String), Vec<MossPair>> = HashMap::new();

    for p in pairs {
        let ua = p
            .file1
            .username
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());
        let ub = p
            .file2
            .username
            .clone()
            .unwrap_or_else(|| "<unknown>".to_string());

        if ua == ub {
            continue;
        }

        let (a, b, p_norm) = if ua <= ub {
            (ua, ub, p)
        } else {
            (
                ub,
                ua,
                MossPair {
                    file1: p.file2.clone(),
                    file2: p.file1.clone(),
                    lines_matched: p.lines_matched,
                    match_href: p.match_href.clone(),
                },
            )
        };
        by_users.entry((a, b)).or_default().push(p_norm);
    }

    let mut reports: Vec<UserPairReport> = Vec::new();
    for ((user_a, user_b), vecp) in by_users.into_iter() {
        let mut submission_id_a: Option<i64> = None;
        let mut submission_id_b: Option<i64> = None;
        let mut total_lines_matched: i64 = 0;

        let mut weighted_sum: f64 = 0.0;
        let mut weight_lines: i64 = 0;

        let mut rows: Vec<FileMatchRow> = Vec::new();
        for p in vecp {
            if submission_id_a.is_none() {
                submission_id_a = p.file1.submission_id;
            }
            if submission_id_b.is_none() {
                submission_id_b = p.file2.submission_id;
            }
            total_lines_matched += p.lines_matched;

            let a_filename = p
                .file1
                .filename
                .clone()
                .unwrap_or_else(|| p.file1.raw.clone());
            let b_filename = p
                .file2
                .filename
                .clone()
                .unwrap_or_else(|| p.file2.raw.clone());

            let percent = match (p.file1.percent, p.file2.percent) {
                (Some(x), Some(y)) => Some(std::cmp::max(x, y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None,
            };

            if let Some(pct) = percent {
                weighted_sum += (pct as f64) * (p.lines_matched as f64);
                weight_lines += p.lines_matched;
            }

            if include_matches {
                rows.push(FileMatchRow {
                    a_filename,
                    b_filename,
                    percent,
                    lines_matched: p.lines_matched,
                    match_href: p.match_href,
                });
            }
        }

        if include_matches {
            rows.sort_by(|x, y| {
                y.lines_matched
                    .cmp(&x.lines_matched)
                    .then(x.a_filename.cmp(&y.a_filename))
                    .then(x.b_filename.cmp(&y.b_filename))
            });
        }

        let total_percent = if weight_lines > 0 {
            let v = weighted_sum / (weight_lines as f64);
            Some((v * 10.0).round() / 10.0)
        } else {
            None
        };

        reports.push(UserPairReport {
            user_a,
            user_b,
            submission_id_a,
            submission_id_b,
            total_lines_matched,
            total_percent,
            matches: if include_matches { Some(rows) } else { None },
        });
    }

    reports.sort_by(|a, b| {
        b.total_lines_matched
            .cmp(&a.total_lines_matched)
            .then(a.user_a.cmp(&b.user_a))
            .then(a.user_b.cmp(&b.user_b))
    });

    reports
}