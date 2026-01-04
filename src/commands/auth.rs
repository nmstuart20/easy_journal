use crate::config::Config;
use crate::error::{JournalError, Result};
use crate::journal::oauth;

pub async fn run(config: &Config) -> Result<()> {
    println!("🔐 Google Tasks Authentication Setup\n");

    // Check if credentials are configured
    let client_id = config.google_oauth.client_id.as_ref().ok_or_else(|| {
        JournalError::OAuthConfigMissing(
            "GOOGLE_CLIENT_ID environment variable not set.\n\
             Please follow setup instructions at: https://developers.google.com/tasks/quickstart/rust"
                .to_string(),
        )
    })?;

    let client_secret = config.google_oauth.client_secret.as_ref().ok_or_else(|| {
        JournalError::OAuthConfigMissing(
            "GOOGLE_CLIENT_SECRET environment variable not set.\n\
             Please follow setup instructions at: https://developers.google.com/tasks/quickstart/rust"
                .to_string(),
        )
    })?;

    println!("📱 Opening browser for authentication...");
    println!("   If browser doesn't open, copy the URL from the terminal.\n");

    // Run OAuth flow
    oauth::authenticate_google(
        client_id.clone(),
        client_secret.clone(),
        &config.google_oauth.token_storage_path,
    )
    .await?;

    println!("\n✨ Setup complete! You can now use Google Tasks in your journal.");
    println!(
        "   Token stored at: {}",
        config.google_oauth.token_storage_path.display()
    );

    Ok(())
}
