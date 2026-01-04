use std::path::Path;
use yup_oauth2::authenticator::Authenticator;
use yup_oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

use crate::error::{JournalError, Result};

/// Run OAuth flow and store refresh token
pub async fn authenticate_google(
    client_id: String,
    client_secret: String,
    token_storage_path: &Path,
) -> Result<()> {
    let app_secret = yup_oauth2::ApplicationSecret {
        client_id,
        client_secret,
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        ..Default::default()
    };

    let auth =
        InstalledFlowAuthenticator::builder(app_secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk(token_storage_path)
            .build()
            .await
            .map_err(|e| {
                JournalError::OAuthFailed(format!("Failed to build authenticator: {}", e))
            })?;

    // Request scope for Google Tasks (read-only)
    auth.token(&["https://www.googleapis.com/auth/tasks.readonly"])
        .await
        .map_err(|e| JournalError::OAuthFailed(format!("Failed to get token: {}", e)))?;

    println!(
        "✅ Authentication successful! Tokens stored at: {}",
        token_storage_path.display()
    );
    Ok(())
}

/// Load existing authenticator from stored tokens
pub async fn load_authenticator(
    client_id: String,
    client_secret: String,
    token_storage_path: &Path,
) -> Result<
    Authenticator<
        yup_oauth2::hyper_rustls::HttpsConnector<yup_oauth2::hyper::client::HttpConnector>,
    >,
> {
    let app_secret = yup_oauth2::ApplicationSecret {
        client_id,
        client_secret,
        auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
        token_uri: "https://oauth2.googleapis.com/token".to_string(),
        ..Default::default()
    };

    InstalledFlowAuthenticator::builder(app_secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk(token_storage_path)
        .build()
        .await
        .map_err(|e| JournalError::OAuthFailed(format!("Failed to load authenticator: {}", e)))
}
