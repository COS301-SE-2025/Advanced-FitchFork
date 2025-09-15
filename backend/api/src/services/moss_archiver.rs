use anyhow::{Context, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use regex::Regex;
use reqwest::{redirect, Client, Url};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{fs as tfs, io::AsyncWriteExt, sync::Mutex};
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedFile {
    pub original_url: String,
    pub saved_rel_path: String,
    pub bytes: u64,
    pub content_type: Option<String>,
}

/// Minimal manifest: just exposes the archive root (relative to `dest_root`)
#[derive(Debug, Serialize, Deserialize)]
pub struct MossArchive {
    pub root_rel: String,
}

#[derive(Clone, Debug)]
pub struct ArchiveOptions {
    /// Max concurrency for HTTP GETs.
    pub concurrency: usize,
}

impl Default for ArchiveOptions {
    fn default() -> Self {
        Self { concurrency: 12 }
    }
}

/// Archive an entire MOSS result set to a local folder.
/// - Mirrors the index, match pages (and their frames), and **all images/assets**.
/// - Rewrites links so the archive works completely offline with `index.html`.
pub async fn archive_moss_to_fs(
    index_url: &str,
    dest_root: &Path,
    opts: ArchiveOptions,
) -> Result<MossArchive> {
    // 1) HTTP client
    let client = Client::builder()
        .user_agent(concat!(
            "fitchfork-moss-archiver/0.1 ",
            "(https://example.invalid) reqwest/"
        ))
        .gzip(true)
        .brotli(true)
        .deflate(true)
        .http1_only()
        .redirect(redirect::Policy::limited(10))
        .build()?;

    // Ensure root dir exists
    tfs::create_dir_all(dest_root).await?;

    // 2) Fetch + save raw index
    let index_abs = Url::parse(index_url).context("invalid index URL")?;
    let (index_html_raw, index_ct) = fetch_text(&client, &index_abs).await?;
    let (match_urls, img_urls_in_index) = extract_index_links(&index_abs, &index_html_raw)?;

    // Shared bookkeeping (original_url -> SavedFile)
    let files = Arc::new(Mutex::new(BTreeMap::<String, SavedFile>::new()));

    // Always download images
    let asset_urls = Arc::new(Mutex::new({
        let mut s = BTreeSet::<Url>::new();
        s.extend(img_urls_in_index.into_iter());
        s
    }));

    // Save index temporarily (raw)
    let index_rel = "index.html".to_string();
    let index_path = dest_root.join(&index_rel);
    write_text(&index_path, &index_html_raw).await?;
    {
        let mut guard = files.lock().await;
        guard.insert(
            index_abs.as_str().to_string(),
            SavedFile {
                original_url: index_abs.as_str().to_string(),
                saved_rel_path: index_rel.clone(),
                bytes: index_html_raw.len() as u64,
                content_type: index_ct,
            },
        );
    }

    // 3) Download all match pages (+ their frames), concurrently
    let unique_match_urls: BTreeSet<Url> = match_urls.into_iter().collect();
    let mut match_index_to_local: BTreeMap<String, String> = BTreeMap::new();

    let sem = Arc::new(tokio::sync::Semaphore::new(opts.concurrency));
    let mut futs = FuturesUnordered::new();

    for murl in unique_match_urls.iter().cloned() {
        let client = client.clone();
        let dest_root = dest_root.to_path_buf();
        let files = Arc::clone(&files);
        let asset_urls = Arc::clone(&asset_urls);
        let sem = Arc::clone(&sem);

        futs.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            download_one_match(&client, &murl, &dest_root, files, asset_urls).await
        }));
    }

    while let Some(res) = futs.next().await {
        let (match_url, local_rel) = res??;
        match_index_to_local.insert(match_url.as_str().to_string(), local_rel);
    }

    // 4) Download assets (images etc.) concurrently — ALWAYS
    let to_fetch: Vec<Url> = {
        let guard = asset_urls.lock().await;
        guard.iter().cloned().collect()
    };

    if !to_fetch.is_empty() {
        let mut afuts = FuturesUnordered::new();
        for aurl in to_fetch {
            let client = client.clone();
            let dest_root = dest_root.to_path_buf();
            let files = Arc::clone(&files);
            let sem = Arc::clone(&sem);

            afuts.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                download_asset(&client, &aurl, &dest_root, files).await
            }));
        }
        while let Some(res) = afuts.next().await {
            res??;
        }
    }

    // 5) Rewrite index to local links (anchors + images) and force a local <base>
    let url_to_rel_asset: BTreeMap<String, String> = {
        let guard = files.lock().await;
        guard
            .iter()
            .map(|(k, v)| (k.clone(), v.saved_rel_path.clone()))
            .collect()
    };

    let index_html_rewritten = rewrite_index_links(
        &index_abs,
        &read_to_string(&index_path).await?,
        &match_index_to_local,
        &url_to_rel_asset,
    )?;
    write_text(&index_path, &index_html_rewritten).await?;

    // 6) Minimal manifest
    Ok(MossArchive { root_rel: ".".to_string() })
}

/* ---------------- ZIP HELPERS ---------------- */

/// Sync zipper: zips everything under `src_dir` into `zip_path`.
/// Keeps paths relative to `src_dir`, with forward slashes in the ZIP.
pub fn zip_dir_sync(src_dir: &Path, zip_path: &Path) -> Result<()> {
    if let Some(parent) = zip_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let file = fs::File::create(zip_path)
        .with_context(|| format!("create zip at {}", zip_path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Skip the root dir itself
        let rel = match path.strip_prefix(src_dir).ok() {
            Some(r) if !r.as_os_str().is_empty() => r,
            _ => continue,
        };

        let rel_str = rel.to_string_lossy().replace('\\', "/");

        if entry.file_type().is_dir() {
            let dir_name = if rel_str.ends_with('/') { rel_str.clone() } else { format!("{}/", rel_str) };
            zip.add_directory(dir_name, options.clone())
                .with_context(|| format!("add dir {}", rel.display()))?;
        } else {
            zip.start_file(rel_str.clone(), options.clone())
                .with_context(|| format!("start file {}", rel.display()))?;
            let bytes = fs::read(path).with_context(|| format!("read file {}", path.display()))?;
            zip.write_all(&bytes)
                .with_context(|| format!("write file {}", rel.display()))?;
        }
    }

    zip.finish().context("finalize zip")?;
    Ok(())
}

/// Async wrapper around `zip_dir_sync` (runs in blocking thread).
pub async fn zip_dir(src_dir: &Path, zip_path: &Path) -> Result<()> {
    let src = src_dir.to_path_buf();
    let zip = zip_path.to_path_buf();
    tokio::task::spawn_blocking(move || zip_dir_sync(&src, &zip))
        .await
        .context("join zip blocking task")?
}

/// Convenience: archive to `dest_root`, then zip to `zip_path`.
/// Returns `(MossArchive, absolute_zip_path)`.
pub async fn archive_moss_to_fs_and_zip(
    index_url: &str,
    dest_root: &Path,
    zip_path: &Path,
    opts: ArchiveOptions,
) -> Result<(MossArchive, String)> {
    let manifest = archive_moss_to_fs(index_url, dest_root, opts).await?;
    zip_dir(dest_root, zip_path).await?;
    let abs = if zip_path.is_absolute() {
        zip_path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(zip_path)
    };
    Ok((manifest, abs.to_string_lossy().to_string()))
}

/* ---------------------------- helpers ---------------------------- */

fn ensure_local_base(mut html: String) -> String {
    // Replace any existing <base ...> with a local base
    let has_base = html.to_lowercase().contains("<base");
    if has_base {
        let re = Regex::new(r"(?is)<base\b[^>]*>").unwrap();
        html = re.replace(&html, r#"<base href="./">"#).to_string();
        return html;
    }

    // insert one after <head> if no base exists
    let re_head = Regex::new(r"(?is)<head\s*>").unwrap();
    if re_head.is_match(&html) {
        return re_head.replace(&html, r#"<head><base href="./">"#).to_string();
    }

    // frameset-only pages (no <head>): inject before first <frameset>
    let re_fs = Regex::new(r"(?is)<frameset\b").unwrap();
    if re_fs.is_match(&html) {
        return re_fs
            .replace(&html, r#"<base href="./"><frameset"#)
            .to_string();
    }

    html
}

async fn fetch_text(client: &Client, url: &Url) -> Result<(String, Option<String>)> {
    let resp = client.get(url.clone()).send().await?.error_for_status()?;
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = resp.text().await?;
    Ok((body, ct))
}

async fn fetch_bytes(client: &Client, url: &Url) -> Result<(bytes::Bytes, Option<String>)> {
    let resp = client.get(url.clone()).send().await?.error_for_status()?;
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let body = resp.bytes().await?;
    Ok((body, ct))
}

async fn write_text(path: &Path, s: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        tfs::create_dir_all(parent).await?;
    }
    let mut f = tfs::File::create(path).await?;
    f.write_all(s.as_bytes()).await?;
    Ok(())
}

async fn write_bytes(path: &Path, b: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tfs::create_dir_all(parent).await?;
    }
    let mut f = tfs::File::create(path).await?;
    f.write_all(b).await?;
    Ok(())
}

async fn read_to_string(path: &Path) -> Result<String> {
    Ok(tfs::read_to_string(path).await?)
}

fn sanitize_file_component(s: &str) -> String {
    let replaced = s.replace('\\', "/");
    let trimmed  = replaced.trim_matches('/');

    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => out.push(ch),
            _ => out.push('_'),
        }
    }

    if out.is_empty() { "_".to_string() } else { out }
}

fn assets_rel_dir() -> &'static str {
    "assets"
}

fn matches_rel_dir() -> &'static str {
    "matches"
}

fn url_basename_sanitized(u: &Url) -> String {
    let seg = u
        .path_segments()
        .and_then(|it| it.last())
        .unwrap_or("match.html");

    let replaced = seg.replace('\\', "/");
    let trimmed  = replaced.trim_matches('/');
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => out.push(ch),
            _ => out.push('_'),
        }
    }
    if out.is_empty() { "_".into() } else { out }
}

fn asset_filename_for_url(u: &Url) -> String {
    // stable hash of URL + extension from URL path (fallback .bin)
    let mut h = Sha256::new();
    h.update(u.as_str().as_bytes());
    let digest = hex::encode(h.finalize());

    let ext = Path::new(u.path())
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    format!("{digest}.{ext}")
}

/* --------- index parsing & rewriting (table of matches + images) --------- */

fn extract_index_links(index_url: &Url, html: &str) -> Result<(Vec<Url>, Vec<Url>)> {
    let doc = Html::parse_document(html);
    let a_sel = Selector::parse("a[href]").unwrap();
    let img_sel = Selector::parse("img[src]").unwrap();

    let mut match_urls = Vec::<Url>::new();
    for a in doc.select(&a_sel) {
        if let Some(href) = a.value().attr("href") {
            if href.contains("match") && href.ends_with(".html") {
                let abs = index_url.join(href).with_context(|| href.to_string())?;
                match_urls.push(abs);
            }
        }
    }

    let mut img_urls = Vec::<Url>::new();
    for img in doc.select(&img_sel) {
        if let Some(src) = img.value().attr("src") {
            let abs = if let Ok(u) = Url::parse(src) { u } else { index_url.join(src)? };
            img_urls.push(abs);
        }
    }

    Ok((match_urls, img_urls))
}

fn rewrite_index_links(
    index_url: &Url,
    original_html: &str,
    match_map: &BTreeMap<String, String>,       // remote match URL -> local rel
    url_to_rel_asset: &BTreeMap<String, String>, // remote asset URL -> local rel
) -> Result<String> {
    let doc = Html::parse_document(original_html);
    let a_sel = Selector::parse("a[href]").unwrap();
    let img_sel = Selector::parse("img[src]").unwrap();

    let mut out = original_html.to_string();

    // a[href] → local match files
    for a in doc.select(&a_sel) {
        if let Some(href) = a.value().attr("href") {
            if href.contains("match") && href.ends_with(".html") {
                let abs = index_url.join(href)?;
                if let Some(local_rel) = match_map.get(abs.as_str()) {
                    out = out.replace(href, local_rel);
                }
            }
        }
    }

    // img[src] on the INDEX page → local assets (index is at root, so "assets/...")
    for img in doc.select(&img_sel) {
        if let Some(src) = img.value().attr("src") {
            let abs = if let Ok(u) = Url::parse(src) { u } else { index_url.join(src)? };
            if let Some(local_rel) = url_to_rel_asset.get(abs.as_str()) {
                out = out.replace(src, local_rel); // e.g., "assets/<hash>.gif"
            }
        }
    }

    Ok(ensure_local_base(out))
}

/* ------------------ download one match & its frames ------------------ */

pub async fn download_one_match(
    client: &Client,
    murl: &Url,
    dest_root: &Path,
    files: Arc<Mutex<BTreeMap<String, SavedFile>>>,
    asset_urls: Arc<Mutex<BTreeSet<Url>>>,
) -> Result<(Url, String)> {
    // 1) Fetch the match page
    let (match_html_raw, ct) = fetch_text(client, murl).await?;

    // 2) Parse frames & images (pure, no await)
    let (frame_urls, img_urls_from_match) = extract_frames_and_images(murl, &match_html_raw)?;

    // 3) Precompute local names for frames:
    //    - html_name: basename only (used inside match page)
    //    - save_rel : "matches/<basename>" (actual saved path)
    let mut frame_url_to_html_name: BTreeMap<String, String> = BTreeMap::new();
    let mut frame_url_to_save_rel:  BTreeMap<String, String> = BTreeMap::new();

    for furl in &frame_urls {
        let base = url_basename_sanitized(furl); // e.g. "match0-0.html"
        frame_url_to_html_name.insert(furl.as_str().to_string(), base.clone());
        frame_url_to_save_rel.insert(
            furl.as_str().to_string(),
            format!("{}/{}", matches_rel_dir(), base),
        );
    }

    // 4) Rewrite the match HTML (no await in this scope)
    let match_html_rewritten = {
        let frame_sel = Selector::parse("frame[src], FRAME[src]").unwrap();
        let img_sel = Selector::parse("img[src], IMG[src]").unwrap();
        let doc = Html::parse_document(&match_html_raw);

        let mut out = match_html_raw.clone();

        // <frame src>: use basename only (since the match page lives in /matches)
        for f in doc.select(&frame_sel) {
            if let Some(src) = f.value().attr("src") {
                let abs = murl.join(src)?;
                if let Some(html_name) = frame_url_to_html_name.get(abs.as_str()) {
                    out = out.replace(src, html_name);
                } else {
                    // If it was "matches/foo.html" → "foo.html"
                    if let Some(last) = Url::parse("https://dummy/").unwrap().join(src).ok()
                        .and_then(|u| u.path_segments().and_then(|p| p.last()).map(|s| s.to_string()))
                    {
                        out = out.replace(src, &last);
                    }
                }
            }
        }

        // <img src> inside a match page (saved under /matches) must point to ../assets/...
        for im in doc.select(&img_sel) {
            if let Some(src) = im.value().attr("src") {
                let abs = if let Ok(u) = Url::parse(src) { u } else { murl.join(src)? };
                let local_name = asset_filename_for_url(&abs);
                out = out.replace(src, &format!("../{}/{}", assets_rel_dir(), local_name));
            }
        }

        ensure_local_base(out)
    }; // drop DOM before any await

    // 5) Queue images referenced by the match page
    {
        let mut aset = asset_urls.lock().await;
        for u in img_urls_from_match {
            aset.insert(u);
        }
    }

    // 6) Save rewritten match page → /matches/<file>
    let file_name = {
        let seg = murl
            .path_segments()
            .and_then(|it| it.last().map(|s| s.to_string()))
            .unwrap_or_else(|| "match.html".to_string());
        sanitize_file_component(&seg)
    };
    let local_rel = format!("{}/{}", matches_rel_dir(), file_name);
    let local_path = dest_root.join(&local_rel);
    write_text(&local_path, &match_html_rewritten).await?;

    // 7) Record match page
    {
        let mut guard = files.lock().await;
        guard.insert(
            murl.as_str().to_string(),
            SavedFile {
                original_url: murl.as_str().to_string(),
                saved_rel_path: local_rel.clone(),
                bytes: fs::metadata(&local_path).map(|m| m.len()).unwrap_or(0),
                content_type: ct.clone(),
            },
        );
    }

    // 8) Download each frame
    for furl in frame_urls {
        let (fhtml, fct) = fetch_text(client, &furl).await?;

        // Rewrite frame HTML (no await)
        let (rewritten, imgs) = rewrite_frame_html(&furl, &fhtml)?;
        let rewritten = ensure_local_base(rewritten);

        // Queue images found in frame
        {
            let mut aset = asset_urls.lock().await;
            for u in imgs {
                aset.insert(u);
            }
        }

        // Save frame to matches/<basename>
        let f_rel = frame_url_to_save_rel.get(furl.as_str()).unwrap().clone();
        let f_path = dest_root.join(&f_rel);
        write_text(&f_path, &rewritten).await?;

        // Record frame file
        {
            let mut guard = files.lock().await;
            guard.insert(
                furl.as_str().to_string(),
                SavedFile {
                    original_url: furl.as_str().to_string(),
                    saved_rel_path: f_rel,
                    bytes: fs::metadata(&f_path).map(|m| m.len()).unwrap_or(0),
                    content_type: fct,
                },
            );
        }
    }

    Ok((murl.clone(), local_rel))
}

fn extract_frames_and_images(match_url: &Url, match_html: &str) -> Result<(Vec<Url>, Vec<Url>)> {
    let doc = Html::parse_document(match_html);
    let frame_sel = Selector::parse("frame[src], FRAME[src]").unwrap();
    let img_sel = Selector::parse("img[src], IMG[src]").unwrap();

    let mut frames = Vec::<Url>::new();
    for f in doc.select(&frame_sel) {
        if let Some(src) = f.value().attr("src") {
            let abs = match_url.join(src)?;
            frames.push(abs);
        }
    }

    let mut imgs = Vec::<Url>::new();
    for im in doc.select(&img_sel) {
        if let Some(src) = im.value().attr("src") {
            let abs = if let Ok(u) = Url::parse(src) { u } else { match_url.join(src)? };
            imgs.push(abs);
        }
    }

    Ok((frames, imgs))
}

/// Rewrites a single frame HTML:
///  - collects <img src> to download
///  - changes <img src> to **../assets/<sha256(url)>.{ext}** (because frame lives in /matches)
fn rewrite_frame_html(frame_url: &Url, html: &str) -> Result<(String, Vec<Url>)> {
    let doc = Html::parse_document(html);
    let img_sel = Selector::parse("img[src]").unwrap();
    let mut out = html.to_string();
    let mut imgs = Vec::<Url>::new();

    for img in doc.select(&img_sel) {
        if let Some(src) = img.value().attr("src") {
            let abs = if let Ok(u) = Url::parse(src) { u } else { frame_url.join(src)? };
            let local_name = asset_filename_for_url(&abs);
            out = out.replace(src, &format!("../{}/{}", assets_rel_dir(), local_name));
            imgs.push(abs);
        }
    }

    Ok((out, imgs))
}

// download_asset: fetches an image/asset and saves it using the **same file name
// scheme** used by HTML rewrites: assets/<sha256(url)>.{ext-from-url}. This
// guarantees the rewritten HTML references match the saved files exactly.
pub async fn download_asset(
    client: &Client,
    aurl: &Url,
    dest_root: &Path,
    files: Arc<Mutex<BTreeMap<String, SavedFile>>>,
) -> Result<()> {
    let (bytes, ct) = fetch_bytes(client, aurl).await?;

    let file_name = asset_filename_for_url(aurl);
    let rel = format!("{}/{}", assets_rel_dir(), file_name);
    let path = dest_root.join(&rel);
    write_bytes(&path, &bytes).await?;

    {
        let mut guard = files.lock().await;
        guard.insert(
            aurl.as_str().to_string(),
            SavedFile {
                original_url: aurl.as_str().to_string(),
                saved_rel_path: rel,
                bytes: bytes.len() as u64,
                content_type: ct,
            },
        );
    }

    Ok(())
}
