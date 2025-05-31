use lettre::{
    message::{header, Message, MultiPart, SinglePart},
    transport::smtp::{authentication::Credentials, AsyncSmtpTransport},
    AsyncTransport, Tokio1Executor,
};
use lettre::transport::smtp::client::{Tls, TlsParameters};
use once_cell::sync::Lazy;
use std::env;

static SMTP_CLIENT: Lazy<AsyncSmtpTransport<Tokio1Executor>> = Lazy::new(|| {
    let username = env::var("GMAIL_USERNAME").expect("GMAIL_USERNAME must be set");
    let password = env::var("GMAIL_APP_PASSWORD").expect("GMAIL_APP_PASSWORD must be set");
    println!("Initializing SMTP client with username: {}", username);

    // Create TLS parameters
    let tls_parameters = TlsParameters::new("smtp.gmail.com".to_string())
        .expect("Failed to create TLS parameters");

    AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .expect("Failed to create SMTP transport")
        .port(587)
        .tls(Tls::Required(tls_parameters))  // Use Required variant
        .credentials(Credentials::new(username, password))
        .build()
});

pub struct EmailService;

impl EmailService {
    pub async fn send_password_reset_email(
        to_email: &str,
        reset_token: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Attempting to send password reset email to: {}", to_email);
        
        let frontend_url = env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
        let from_email = env::var("GMAIL_USERNAME").expect("GMAIL_USERNAME must be set");
        let from_name = env::var("EMAIL_FROM_NAME").expect("EMAIL_FROM_NAME must be set");
        
        println!("Using frontend URL: {}", frontend_url);
        println!("From email: {}", from_email);
        println!("From name: {}", from_name);
        
        let reset_link = format!("{}/reset-password?token={}", frontend_url, reset_token);
        println!("Generated reset link: {}", reset_link);

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

        println!("Attempting to send email...");
        match SMTP_CLIENT.send(email).await {
            Ok(_) => {
                println!("Email sent successfully!");
                Ok(())
            }
            Err(e) => {
                println!("Failed to send email: {}", e);
                Err(Box::new(e) as Box<dyn std::error::Error>)
            }
        }
    }
}