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

    pub async fn send_email_when_spec_changes(
        to_email: Vec<String>,
        spec_name: String,
        change_details: String,
    ) {
        let from_email = config::gmail_username();
        let from_name = config::email_from_name();

        let mut builder =
            Message::builder().from(format!("{} <{}>", from_name, from_email).parse().unwrap());

        for recipient in to_email {
            builder = builder.to(recipient.parse().unwrap());
        }

        let email_result = builder
            .subject(format!("Specification '{}' Has Changed", spec_name))
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(header::ContentType::TEXT_PLAIN)
                            .body(format!(
                                "Hello,\n\n\
                                The specification '{}' has been updated. Here are the details of the changes:\n\n\
                                {}\n\n\
                                Please review the changes at your earliest convenience.\n\n\
                                Best regards,\n\
                                {}",
                                spec_name,
                                change_details,
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
                                <p>The specification '<strong>{}</strong>' has been updated. Here are the details of the changes:</p>\
                                <pre>{}</pre>\
                                <p>Please review the changes at your earliest convenience.</p>\
                                <p>Best regards,<br>\
                                {}</p>\
                                </body>\
                                </html>",
                                spec_name,
                                change_details,
                                from_name
                            )),
                    ),
            );

        let email = match email_result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to build email: {}", e);
                return;
            }
        };

        match SMTP_CLIENT.send(email).await {
            Ok(_) => (),
            Err(e) => eprintln!("Failed to send email: {}", e),
        };
    }
}
