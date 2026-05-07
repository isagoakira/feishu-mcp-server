/// OAuth 2.0 token management for Feishu API
/// 获取和刷新 tenant_access_token

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Feishu API response for tenant_access_token
#[derive(Debug, Deserialize)]
struct TokenResponse {
    code: u32,
    msg: String,
    tenant_access_token: Option<String>,
    expire: Option<u64>, // seconds until expiration
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantAccessToken {
    pub token: String,
    pub expire: u64, // Unix timestamp when token expires
}

impl TenantAccessToken {
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp() as u64;
        self.expire <= now
    }
}

#[derive(Debug)]
pub struct OAuthManager {
    app_id: String,
    app_secret: String,
    api_base: String,
    client: Client,
    current_token: Arc<RwLock<Option<TenantAccessToken>>>,
}

impl OAuthManager {
    pub fn new(app_id: String, app_secret: String, api_base: String) -> Self {
        Self {
            app_id,
            app_secret,
            api_base,
            client: Client::new(),
            current_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Fetch new tenant_access_token from Feishu API
    pub async fn fetch_token(&self) -> anyhow::Result<TenantAccessToken> {
        let url = format!("{}/auth/v3/tenant_access_token/internal", self.api_base);

        let params = serde_json::json!({
            "app_id": self.app_id,
            "app_secret": self.app_secret
        });

        tracing::debug!("Fetching token from Feishu API");

        let response = self.client
            .post(&url)
            .json(&params)
            .send()
            .await?;

        let token_resp: TokenResponse = response.json().await?;

        if token_resp.code != 0 {
            tracing::error!("Feishu API error: code={}, msg={}", token_resp.code, token_resp.msg);
            anyhow::bail!("Failed to get token: {} - {}", token_resp.code, token_resp.msg);
        }

        let token = token_resp.tenant_access_token
            .ok_or_else(|| anyhow::anyhow!("Missing tenant_access_token in response"))?;

        let expire_seconds = token_resp.expire.unwrap_or(7200);
        let now = chrono::Utc::now().timestamp() as u64;
        let expire_timestamp = now + expire_seconds;

        tracing::info!("Successfully obtained token, expires in {} seconds", expire_seconds);

        Ok(TenantAccessToken {
            token,
            expire: expire_timestamp,
        })
    }

    /// Get current token or refresh if expired
    pub async fn get_or_refresh_token(&self) -> anyhow::Result<TenantAccessToken> {
        let token = self.current_token.read().await;
        if let Some(t) = &*token {
            if !t.is_expired() {
                tracing::debug!("Using cached token (expires at {})", t.expire);
                return Ok(t.clone());
            }
        }
        drop(token);

        tracing::debug!("Token expired or missing, fetching new token");
        let new_token = self.fetch_token().await?;
        let mut token = self.current_token.write().await;
        *token = Some(new_token.clone());
        Ok(new_token)
    }

    /// Manually set the token (for loading from store)
    pub async fn set_token(&self, token: TenantAccessToken) {
        let mut t = self.current_token.write().await;
        *t = Some(token);
    }
}

// Manually implement Clone since reqwest::Client doesn't derive Clone
impl Clone for OAuthManager {
    fn clone(&self) -> Self {
        Self {
            app_id: self.app_id.clone(),
            app_secret: self.app_secret.clone(),
            api_base: self.api_base.clone(),
            client: self.client.clone(),
            current_token: Arc::new(RwLock::new(None)),
        }
    }
}