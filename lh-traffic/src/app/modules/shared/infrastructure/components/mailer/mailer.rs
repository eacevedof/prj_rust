use anyhow::{Result, Context};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;

use crate::{log_info, log_error};

/// Mailer component for sending emails via SMTP using cURL
///
/// Compatible with Deno implementation - uses cURL for maximum flexibility
/// and portability across different systems.
///
/// Usage (builder pattern):
/// ```rust
/// let result = Mailer::new()
///     .set_subject("Test Email")
///     .set_email_to("user@example.com")
///     .set_email_from("noreply@example.com")
///     .set_message("<h1>Hello</h1>")
///     .send_template()
///     .await?;
/// ```
#[derive(Clone)]
pub struct Mailer {
    email_from: String,
    email_from_name: String,
    email_to: String,
    emails_cc: Vec<String>,
    subject: String,
    message: String,
    smtp_config: Option<SmtpConfig>,
    sent_result: Option<MailSentResult>,
    path_log_email_file: String,
    path_tmp_file: String,
}

#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub email_from: String,
    pub email_from_name: String,
    pub protocol: String, // "smtp" or "smtps"
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
}

#[derive(Debug, Clone)]
pub struct MailSentResult {
    pub success: bool,
    pub error: Option<String>,
    pub tmp_random_file: String,
}

impl Mailer {
    /// Create a new Mailer instance
    pub fn new() -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let path_log_email_file = format!("/tmp/email-{}.log", today);

        Self {
            email_from: String::new(),
            email_from_name: String::new(),
            email_to: String::new(),
            emails_cc: Vec::new(),
            subject: String::new(),
            message: String::new(),
            smtp_config: None,
            sent_result: None,
            path_log_email_file,
            path_tmp_file: String::new(),
        }
    }

    /// Get a new instance (alias for new, matches Deno API)
    pub fn get_instance() -> Self {
        Self::new()
    }

    // ========================================================================
    // BUILDER METHODS (fluent API)
    // ========================================================================

    pub fn set_subject(mut self, subject: &str) -> Self {
        self.subject = subject.to_string();
        self
    }

    pub fn set_email_to(mut self, email_to: &str) -> Self {
        self.email_to = email_to.to_string();
        self
    }

    pub fn set_email_from(mut self, email_from: &str) -> Self {
        self.email_from = email_from.to_string();
        self
    }

    pub fn set_email_from_name(mut self, email_from_name: &str) -> Self {
        self.email_from_name = email_from_name.to_string();
        self
    }

    pub fn add_email_cc(mut self, email_cc: &str) -> Self {
        self.emails_cc.push(email_cc.to_string());
        self
    }

    pub fn set_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn set_smtp_config(mut self, config: SmtpConfig) -> Self {
        self.smtp_config = Some(config);
        self
    }

    pub fn set_tpl_view(mut self, tpl_vars: HashMap<String, String>) -> Self {
        // Extract subject from vars if not set
        if self.subject.is_empty() {
            if let Some(subject) = tpl_vars.get("subject") {
                self.subject = subject.clone();
            }
        }
        self
    }

    pub fn set_default_tpl_vars(self, tpl_vars: HashMap<String, String>) -> Self {
        self.set_tpl_view(tpl_vars)
    }

    // ========================================================================
    // SEND METHODS
    // ========================================================================

    /// Send email using template (loads HTML from file and replaces variables)
    pub async fn send_template_file(
        mut self,
        template_path: &str,
        vars: &HashMap<String, String>,
    ) -> Result<Self> {
        // Load template
        let template_content = tokio::fs::read_to_string(template_path)
            .await
            .context("Failed to read template file")?;

        // Replace variables {{key}} with values
        let mut processed = template_content;
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            processed = processed.replace(&placeholder, value);
        }

        // Update subject from vars if not set
        if self.subject.is_empty() {
            if let Some(subject) = vars.get("subject") {
                self.subject = subject.clone();
            }
        }

        self.message = processed;

        self.send_template().await
    }

    /// Send email with current message (main send method)
    pub async fn send_template(mut self) -> Result<Self> {
        self.fail_if_wrong_input()?;

        self.send_email_with_curl().await?;

        Ok(self)
    }

    // ========================================================================
    // RESULT AND RESET
    // ========================================================================

    pub fn get_result(&self) -> MailSentResult {
        self.sent_result.clone().unwrap_or(MailSentResult {
            success: false,
            error: Some("No email sent yet".to_string()),
            tmp_random_file: String::new(),
        })
    }

    pub fn reset(mut self) -> Self {
        self.subject = String::new();
        self.email_to = String::new();
        self.email_from = String::new();
        self.email_from_name = String::new();
        self.emails_cc = Vec::new();
        self.message = String::new();
        self.sent_result = None;
        self
    }

    // ========================================================================
    // PRIVATE HELPER METHODS
    // ========================================================================

    async fn send_email_with_curl(&mut self) -> Result<()> {
        let email_from = self.get_from_email_with_alias();
        let email_to = self.email_to.clone(); // Clone to avoid borrowing issues
        let subject = if !self.subject.is_empty() {
            self.subject.clone()
        } else {
            "Rust Email".to_string()
        };
        let time_now = chrono::Local::now().to_rfc2822();

        // Build CC headers and recipients
        let mut cc_headers = String::new();
        let mut cc_recipients = String::new();

        if !self.emails_cc.is_empty() {
            cc_headers = format!("Cc: {}\n", self.emails_cc.join(", "));
            cc_recipients = self
                .emails_cc
                .iter()
                .map(|cc| format!("--mail-rcpt \"{}\"", cc))
                .collect::<Vec<_>>()
                .join(" ");
        }

        // Build email content
        let message = self.message.clone();
        let email_content = format!(
            "From: {}\nTo: {}\n{}Subject: {}\nContent-Type: text/html; charset=UTF-8\nDate: {}\n\n{}",
            email_from.trim(),
            email_to.trim(),
            cc_headers.trim(),
            subject.trim(),
            time_now,
            message
        );

        // Get cURL command
        let curl_command = self.get_curl_email_command(&email_content, &cc_recipients).await?;

        // Log command
        self.log_command(&curl_command).await?;

        // Execute cURL command
        log_info!("Sending email via cURL to {}", email_to);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&curl_command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute cURL command")?;

        // Check result
        if output.status.success() {
            log_info!("Email sent successfully to {}", email_to);
            self.sent_result = Some(MailSentResult {
                success: true,
                error: None,
                tmp_random_file: self.path_tmp_file.clone(),
            });
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
            log_error!("Failed to send email: {}", error_msg);
            self.sent_result = Some(MailSentResult {
                success: false,
                error: Some(error_msg),
                tmp_random_file: self.path_tmp_file.clone(),
            });
        }

        Ok(())
    }

    async fn get_curl_email_command(&mut self, email_content: &str, cc_recipients: &str) -> Result<String> {
        let mail_smtp_config = self.get_default_email_config();

        let smtp_url = format!(
            "{}://{}:{}",
            mail_smtp_config.protocol, mail_smtp_config.smtp_host, mail_smtp_config.smtp_port
        );

        // Generate random tmp file path
        self.path_tmp_file = self.get_random_tmp_file_path();

        // Write email content to temp file
        tokio::fs::write(&self.path_tmp_file, email_content)
            .await
            .context("Failed to write email content to temp file")?;

        let from_email = if !self.email_from.is_empty() {
            &self.email_from
        } else {
            &mail_smtp_config.email_from
        };

        // Build cURL command (similar to Deno version)
        let nohup_parts = vec![
            format!("nohup sh -c 'curl --verbose --silent --show-error"),
            format!("--max-time 60"),
            format!("--url \"{}\"", smtp_url),
            format!("--user \"{}:{}\"", mail_smtp_config.smtp_user, mail_smtp_config.smtp_pass),
            format!("--mail-from \"{}\"", from_email),
            format!("--mail-rcpt \"{}\"", self.email_to),
            cc_recipients.to_string(),
            format!("--upload-file {}", self.path_tmp_file),
            format!("&& sleep 10 && rm {}' >> {} 2>&1", self.path_tmp_file, self.path_log_email_file),
        ];

        Ok(nohup_parts.join(" "))
    }

    fn get_default_email_config(&self) -> SmtpConfig {
        if let Some(ref config) = self.smtp_config {
            return config.clone();
        }

        SmtpConfig {
            email_from: std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@example.com".to_string()),
            email_from_name: std::env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "System".to_string()),
            protocol: std::env::var("SMTP_PROTOCOL")
                .unwrap_or_else(|_| "smtps".to_string()),
            smtp_host: std::env::var("SMTP_HOST")
                .unwrap_or_else(|_| "smtp.example.com".to_string()),
            smtp_port: std::env::var("SMTP_PORT")
                .unwrap_or_else(|_| "465".to_string())
                .parse()
                .unwrap_or(465),
            smtp_user: std::env::var("SMTP_USER")
                .unwrap_or_else(|_| "user@example.com".to_string()),
            smtp_pass: std::env::var("SMTP_PASS")
                .unwrap_or_else(|_| "password".to_string()),
        }
    }

    fn get_from_email_with_alias(&self) -> String {
        let from_email = self.get_from_email();
        let from_name = self.get_from_name();

        if !from_name.is_empty() && from_name != from_email {
            format!("\"{}\" <{}>", from_name, from_email)
        } else {
            from_email
        }
    }

    fn get_from_email(&self) -> String {
        if !self.email_from.is_empty() {
            return self.email_from.clone();
        }

        let config = self.get_default_email_config();
        if !config.email_from.is_empty() {
            return config.email_from;
        }

        config.smtp_user
    }

    fn get_from_name(&self) -> String {
        if !self.email_from_name.is_empty() {
            return self.email_from_name.clone();
        }

        let config = self.get_default_email_config();
        if !config.email_from_name.is_empty() {
            return config.email_from_name;
        }

        config.smtp_user
    }

    fn get_random_tmp_file_path(&self) -> String {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let now = chrono::Local::now().format("%H%M%S").to_string();

        // Generate random hex string
        use rand::Rng;
        let random: String = (0..10)
            .map(|_| format!("{:02x}", rand::thread_rng().gen::<u8>()))
            .collect();

        format!("/tmp/eml-{}-{}-{}", today, now, random)
    }

    async fn log_command(&self, command: &str) -> Result<()> {
        let log_entry = format!("{}\n", command);

        tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path_log_email_file)
            .await
            .context("Failed to open log file")?
            .write_all(log_entry.as_bytes())
            .await
            .context("Failed to write to log file")?;

        Ok(())
    }

    fn fail_if_wrong_input(&self) -> Result<()> {
        if self.email_to.is_empty() {
            anyhow::bail!("email-to is required");
        }

        if !self.is_valid_email(&self.email_to) {
            anyhow::bail!("email-to {} is not a valid email", self.email_to);
        }

        if self.subject.is_empty() {
            anyhow::bail!("subject is required");
        }

        Ok(())
    }

    fn is_valid_email(&self, email: &str) -> bool {
        let re = regex::Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        re.is_match(email)
    }
}

impl Default for Mailer {
    fn default() -> Self {
        Self::new()
    }
}
