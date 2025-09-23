use reqwest::{Client, StatusCode};
use std::time::Duration;

/// Returns true if the URL appears reachable (2xx/3xx considered alive).
///  - HEAD first (fast), fall back to GET on 405/501.
///  - `timeout_secs` caps the whole request timeout.
pub async fn is_url_alive(url: &str, timeout_secs: u64) -> Result<bool, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    // Try HEAD
    let head = client.head(url).send().await;
    match head {
        Ok(resp) => {
            let code = resp.status();
            if code.is_success() || (code.is_redirection() || code == StatusCode::NOT_MODIFIED) {
                return Ok(true);
            }
            // Some hosts disallow HEAD → use GET fallback if method not allowed
            if code == StatusCode::METHOD_NOT_ALLOWED || code == StatusCode::NOT_IMPLEMENTED {
                // fall through to GET below
            } else {
                return Ok(false);
            }
        }
        Err(_) => {
            // fall through to GET below
        }
    }

    // Fallback: GET
    let get = client.get(url).send().await?;
    let code = get.status();
    Ok(code.is_success() || code.is_redirection() || code == StatusCode::NOT_MODIFIED)
}
