/// SQLite-based encrypted token storage with AES-256-GCM

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const NONCE_SIZE: usize = 12;

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Token store backed by SQLite with AES-256-GCM encryption
pub struct TokenStore {
    #[allow(dead_code)]
    db_path: String,
    cipher: Arc<Aes256Gcm>,
    conn: Arc<Mutex<Connection>>,
}

impl TokenStore {
    /// Create a new TokenStore with the given encryption key
    /// The key must be exactly 32 bytes for AES-256
    pub fn new(db_path: String, encryption_key: &[u8; 32]) -> anyhow::Result<Self> {
        let cipher = Aes256Gcm::new_from_slice(encryption_key)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {:?}", e))?;

        let conn = Connection::open(&db_path)?;

        let store = Self {
            db_path,
            cipher: Arc::new(cipher),
            conn: Arc::new(Mutex::new(conn)),
        };

        store.init_db()?;
        Ok(store)
    }

    /// Initialize SQLite database and tables
    pub fn init_db(&self) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tokens (
                app_id TEXT PRIMARY KEY,
                encrypted_token BLOB NOT NULL,
                expires_at INTEGER NOT NULL,
                refreshed_at INTEGER NOT NULL
            )",
            [],
        )?;
        tracing::debug!("Token store database initialized");
        Ok(())
    }

    /// Encrypt and save token to database
    pub fn save_token(&self, app_id: &str, token: &str, expire_timestamp: u64) -> anyhow::Result<()> {
        // Generate random nonce
        let nonce_bytes: [u8; NONCE_SIZE] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let payload = Payload {
            aad: app_id.as_bytes(),
            msg: token.as_bytes(),
        };

        let ciphertext = self.cipher.encrypt(nonce, payload)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        // Prepend nonce to ciphertext for storage
        let mut stored = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        stored.extend_from_slice(&nonce_bytes);
        stored.extend_from_slice(&ciphertext);

        let now = current_timestamp();

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO tokens (app_id, encrypted_token, expires_at, refreshed_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![app_id, stored, expire_timestamp as i64, now as i64],
        )?;

        tracing::debug!("Token saved for app_id: {}", app_id);
        Ok(())
    }

    /// Load and decrypt token from database
    /// Returns (token, expires_at) if found and not expired
    pub fn load_token(&self, app_id: &str) -> anyhow::Result<Option<(String, u64)>> {
        let conn = self.conn.lock().unwrap();
        let result: rusqlite::Result<(Vec<u8>, i64)> = conn.query_row(
            "SELECT encrypted_token, expires_at FROM tokens WHERE app_id = ?1",
            params![app_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok((stored, expires_at)) => {
                let expires_at = expires_at as u64;

                // Check if expired
                if expires_at <= current_timestamp() {
                    tracing::debug!("Token for {} is expired", app_id);
                    return Ok(None);
                }

                if stored.len() < NONCE_SIZE {
                    anyhow::bail!("Invalid stored token: too short");
                }

                // Extract nonce and ciphertext
                let nonce = Nonce::from_slice(&stored[..NONCE_SIZE]);
                let ciphertext = &stored[NONCE_SIZE..];

                let payload = Payload {
                    aad: app_id.as_bytes(),
                    msg: ciphertext,
                };

                let token = self.cipher.decrypt(nonce, payload)
                    .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

                let token_str = String::from_utf8(token)
                    .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in token: {:?}", e))?;

                tracing::debug!("Token loaded for app_id: {} (expires at {})", app_id, expires_at);
                Ok(Some((token_str, expires_at)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                tracing::debug!("No token found for app_id: {}", app_id);
                Ok(None)
            }
            Err(e) => anyhow::bail!("Failed to load token: {:?}", e),
        }
    }

    /// Delete token from database
    #[allow(dead_code)]
    pub fn delete_token(&self, app_id: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM tokens WHERE app_id = ?1",
            params![app_id],
        )?;
        tracing::debug!("Token deleted for app_id: {}", app_id);
        Ok(())
    }
}