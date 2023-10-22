use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Client {
    /// The address of the recipient
    to: String,

    /// The host name of the sender
    domain: String,

    /// MailGun API base path
    api_base_path: String,

    /// MailGun API key
    api_key: String,
}

impl Client {
    pub fn init_from_file(path: &Path) -> Result<Client> {
        let config = fs::read_to_string(path).context("Failed to read email config file")?;

        toml::from_str::<Client>(&config).context("Failed to parse email config file")
    }

    pub fn send_email(&self, file: Option<&str>, log: &str) -> Result<()> {
        let url = format!(
            "{}/{}/messages",
            self.api_base_path.trim_end_matches('/'),
            self.domain
        );

        let subject = match file {
            Some(file) => format!("Dessert has been served: {}", file),
            None => "Dessert is ruined".to_string(),
        };

        let response = reqwest::blocking::Client::new()
            .post(url)
            .basic_auth("api", Some(&self.api_key))
            .multipart(
                reqwest::blocking::multipart::Form::new()
                    .text("from", "Dessert <dessert@mg.jonstodle.no>")
                    .text("to", self.to.clone())
                    .text("subject", subject)
                    .text(
                        "text",
                        format!(
                            r"{}

                            {log}",
                            file.unwrap_or(""),
                        ),
                    ),
            )
            .send()
            .context("Failed to send email")?;

        if !response.status().is_success() {
            Err(anyhow!("Failed to send email: {}", response.text()?))
        } else {
            Ok(())
        }
    }
}
