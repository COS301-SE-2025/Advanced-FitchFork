//! Email service module for handling email-related functionality.
//!
//! This module provides functionality for sending various types of emails using SMTP,
//! specifically configured for Gmail. It uses the `lettre` crate for email handling
//! and supports both plain text and HTML email formats.
//!
//! # Environment Variables Required
//! - `GMAIL_USERNAME`: Gmail address to send emails from
//! - `GMAIL_APP_PASSWORD`: Gmail app password for authentication
//! - `FRONTEND_URL`: Base URL of the frontend application
//! - `EMAIL_FROM_NAME`: Display name for the sender

use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{
    AsyncTransport, Tokio1Executor,
    message::{Message, MultiPart, SinglePart, header},
    transport::smtp::{AsyncSmtpTransport, authentication::Credentials},
};
use once_cell::sync::Lazy;
use util::config;

/// Global SMTP client instance configured for Gmail.
///
/// This is initialized lazily when first used, using environment variables
/// for configuration. The client is configured to use TLS and requires
/// authentication.
static SMTP_CLIENT: Lazy<AsyncSmtpTransport<Tokio1Executor>> = Lazy::new(|| {
    let username = config::gmail_username();
    let password = config::gmail_app_password();

    let tls_parameters =
        TlsParameters::new("smtp.gmail.com".to_string()).expect("Failed to create TLS parameters");

    AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .expect("Failed to create SMTP transport")
        .port(587)
        .tls(Tls::Required(tls_parameters))
        .credentials(Credentials::new(username, password))
        .build()
});

/// Service for handling email-related operations.
pub struct EmailService;

impl EmailService {
    /// Sends a password reset email to the specified email address.
    ///
    /// # Arguments
    /// * `to_email` - The recipient's email address
    /// * `reset_token` - The password reset token to include in the reset link
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok(()) if email was sent successfully,
    ///   Err containing the error if sending failed
    ///
    /// # Email Content
    /// The email includes both plain text and HTML versions with:
    /// * A personalized greeting
    /// * A reset password link
    /// * Expiration notice (15 minutes)
    /// * Security warning for unintended recipients
    /// * Styled HTML version with a clickable button
    pub async fn send_password_reset_email(
        to_email: &str,
        reset_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let frontend_url = config::frontend_url();
        let from_email = config::gmail_username();
        let from_name = config::email_from_name();
        let reset_link = format!("{}/reset-password?token={}", frontend_url, reset_token);

        let email = Message::builder()
            .from(format!("{} <{}>", from_name, from_email).parse().unwrap())
            .to(to_email.parse().unwrap())
            .subject("Reset Your Password")
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(format!(
                                "Hello,\n\n\
                                You have requested to reset your password. Click the link below to proceed:\n\n\
                                {}\n\n\
                                This link will expire in 15 minutes.\n\n\
                                If you did not request this password reset, please ignore this email.\n\n\
                                Best regards,\n\
                                {}",
                                reset_link,
                                from_name
                            )),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(format!(
                                r#"<!DOCTYPE html>
                                <html>
                                <head>
                                    <style>
                                        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
                                        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; text-align: center; }}
                                        .button {{ 
                                            display: inline-block;
                                            padding: 10px 20px;
                                            background-color: #007bff;
                                            color: #ffffff !important;
                                            text-decoration: none;
                                            border-radius: 5px;
                                            margin: 20px 0;
                                            font-weight: bold;
                                        }}
                                        .warning {{ color: #dc3545; }}
                                    </style>
                                </head>
                                <body>
                                    <div class="container">
                                        <h2>Reset Your Password</h2>
                                        <p>Hello,</p>
                                        <p>You have requested to reset your password. Click the button below to proceed:</p>
                                        <a href="{}" class="button">Reset Password</a>
                                        <p>This link will expire in 15 minutes.</p>
                                        <p class="warning">If you did not request this password reset, please ignore this email.</p>
                                        <p>Best regards,<br>{}</p>
                                    </div>
                                </body>
                                </html>"#,
                                reset_link, from_name
                            )),
                    ),
            )?;

        match SMTP_CLIENT.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
        }
    }

    /// Sends a password change confirmation email to the specified email address.
    ///
    /// # Arguments
    /// * `to_email` - The recipient's email address
    ///
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok(()) if email was sent successfully,
    ///   Err containing the error if sending failed
    ///
    /// # Email Content
    /// The email includes both plain text and HTML versions with:
    /// * Confirmation of password change
    /// * Security warning for unintended changes
    /// * Simple HTML formatting
    pub async fn send_password_changed_email(
        to_email: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let from_email = config::gmail_username();
        let from_name = config::email_from_name();

        let email = Message::builder()
            .from(format!("{} <{}>", from_name, from_email).parse().unwrap())
            .to(to_email.parse().unwrap())
            .subject("Your Password Has Been Changed")
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(format!(
                                "Hello,\n\n\
                                Your password has been successfully changed.\n\n\
                                If you did not make this change, please contact support immediately.\n\n\
                                Best regards,\n\
                                {}",
                                from_name
                            )),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_HTML)
                            .body(format!(
                                "<html>\
                                <body>\
                                <p>Hello,</p>\
                                <p>Your password has been successfully changed.</p>\
                                <p>If you did not make this change, please contact support immediately.</p>\
                                <p>Best regards,<br>\
                                {}</p>\
                                </body>\
                                </html>",
                                from_name
                            )),
                    ),
            )?;

        match SMTP_CLIENT.send(email).await {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
        }
    }

    /// Notify users when an assignment specification file changes.
    ///
    /// `module_code` (e.g. "COS301"), `module_year` (e.g. 2025), `assignment_name` (e.g. "A1: Linked Lists")
    /// `module_id` and `assignment_id` are used to build a deep link back to the UI.
    pub async fn send_spec_change_email(
        to_emails: Vec<String>,
        module_code: &str,
        module_year: i32,
        module_id: i64,
        assignment_id: i64,
        assignment_name: &str,
        spec_filename: &str,
        change_summary: Option<&str>,
    ) {
        // Tiny local helper to avoid extra crates
        fn escape_html(input: &str) -> String {
            let mut out = String::with_capacity(input.len());
            for ch in input.chars() {
                match ch {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '"' => out.push_str("&quot;"),
                    '\'' => out.push_str("&#39;"),
                    _ => out.push(ch),
                }
            }
            out
        }

        if to_emails.is_empty() {
            eprintln!("send_spec_change_email: empty recipient list; skipping send");
            return;
        }

        let from_email = config::gmail_username();
        let from_name = config::email_from_name();
        let frontend = config::frontend_url();

        // Deep link to the assignment (adjust querystring/fragment if you want to land on Files tab)
        let assignment_url = format!(
            "{}/modules/{}/assignments/{}",
            frontend, module_id, assignment_id
        );

        // Subject: "Spec updated: COS301 (2025) — A1: Linked Lists"
        let subject = format!(
            "Spec updated: {} ({}) — {}",
            module_code, module_year, assignment_name
        );

        let summary_text = change_summary.unwrap_or("—");

        // Plain-text part
        let text_body = format!(
            "Specification updated\n\n\
             Module: {} ({})\n\
             Assignment: {}\n\
             File: {}\n\
             Link: {}\n\n\
             Change summary:\n{}\n\n\
             Best regards,\n{}",
            module_code,
            module_year,
            assignment_name,
            spec_filename,
            assignment_url,
            summary_text,
            from_name
        );

        // HTML part
        let html_body = format!(
            r#"<!doctype html>
<html>
<head>
  <meta charset="utf-8" />
  <style>
    body {{ font-family: Arial, sans-serif; color:#333; line-height:1.55; }}
    .container {{ max-width: 640px; margin: 0 auto; padding: 20px; }}
    .h {{ margin: 0 0 12px; }}
    .meta {{ margin: 14px 0; padding:12px; background:#f7f7f9; border:1px solid #eee; border-radius:6px; }}
    .meta dt {{ font-weight:bold; }}
    .btn {{
      display:inline-block; padding:10px 16px; border-radius:6px;
      background:#1677ff; color:#fff !important; text-decoration:none; font-weight:600;
      margin: 10px 0;
    }}
    pre {{ background:#0b1022; color:#f3f7ff; padding:12px; border-radius:6px; overflow:auto; }}
    code {{ font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace; }}
  </style>
</head>
<body>
  <div class="container">
    <h2 class="h">Specification updated</h2>

    <div class="meta">
      <dl>
        <dt>Module</dt><dd>{module_code} ({module_year})</dd>
        <dt>Assignment</dt><dd>{assignment_name}</dd>
        <dt>File</dt><dd><code>{spec_filename}</code></dd>
      </dl>
    </div>

    <p><a class="btn" href="{assignment_url}">Open assignment</a></p>

    <h3 class="h">Change summary</h3>
    <pre>{escaped_summary}</pre>

    <p style="margin-top:16px;">Best regards,<br/>{from_name}</p>
  </div>
</body>
</html>
"#,
            module_code = module_code,
            module_year = module_year,
            assignment_name = assignment_name,
            spec_filename = spec_filename,
            assignment_url = assignment_url,
            escaped_summary = escape_html(summary_text),
            from_name = from_name
        );

        // Build message with multiple recipients
        let mut builder = Message::builder()
            .from(format!("{} <{}>", from_name, from_email).parse().unwrap())
            .subject(subject);

        for rcpt in to_emails {
            builder = builder.to(rcpt.parse().unwrap());
        }

        let msg = match builder.multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(text_body),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        ) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to build spec-change email: {}", e);
                return;
            }
        };

        if let Err(e) = SMTP_CLIENT.send(msg).await {
            eprintln!("Failed to send spec-change email: {}", e);
        }
    }

    pub async fn send_marking_done_email(
        to_email: &str,
        display_name: &str,
        submission_id: i64,
        module_id: i64,
        assignment_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let from_email = config::gmail_username();
        let from_name = config::email_from_name();

        let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse().unwrap())
        .to(to_email.parse().unwrap())
        .subject(format!("Submission #{} – Marking complete", submission_id))
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(format!(
                            "Hello {},\n\n\
                             Your submission #{} for module #{}, assignment #{} has finished marking.\n\n\
                             You can now view your results on FitchFork.\n\n\
                             Best regards,\n\
                             {}",
                            display_name, submission_id, module_id, assignment_id, from_name
                        )),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(format!(
                            "<html>\
                             <body>\
                             <p>Hello {},</p>\
                             <p>Your submission <b>#{}</b> for module <b>#{}</b>, assignment <b>#{}</b> has finished marking.</p>\
                             <p>You can now view your results on FitchFork.</p>\
                             <p>Best regards,<br>\
                             {}</p>\
                             </body>\
                             </html>",
                            display_name, submission_id, module_id, assignment_id, from_name
                        )),
                ),
        )?;

        SMTP_CLIENT
            .send(email)
            .await
            .map(|_| ())
            .map_err(|e| Box::new(e) as _)
    }
}
