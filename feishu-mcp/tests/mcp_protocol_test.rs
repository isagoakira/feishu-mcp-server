/// Integration tests for Feishu MCP Server core components.
///
/// Tests rate limiting, token storage, client construction,
/// and server initialization via public APIs.

use std::sync::Arc;

use feishu_mcp::FeishuMcpServer;
use feishu_mcp::auth::token_store::TokenStore;
use feishu_mcp::feishu::client::FeishuClient;
use feishu_mcp::middleware::rate_limit::RateLimiter;

/// Helper: create an isolated test database path
fn temp_db(name: &str) -> String {
    format!("/tmp/feishu_int_{}_{}.db", name, rand::random::<u64>())
}

/// Helper: create a TestServer with isolated token store
fn create_test_server() -> FeishuMcpServer {
    let db = temp_db("server");
    let _ = std::fs::remove_file(&db);
    let key: [u8; 32] = rand::random();
    let store = TokenStore::new(db, &key).expect("Failed to create token store");
    let client = FeishuClient::new(
        "https://open.feishu.cn/open-apis".to_string(),
        Arc::new(store),
        "test-app-id".to_string(),
    );
    FeishuMcpServer::new(Arc::new(client), Arc::new(RateLimiter::new(60, 1000)))
}

// ============================================================================
// Rate Limiter
// ============================================================================

#[test]
fn test_rate_limiter_basic() {
    let limiter = RateLimiter::new(60, 2);
    assert!(limiter.check("tool").is_ok());
    assert!(limiter.check("tool").is_ok());
    let err = limiter.check("tool").unwrap_err();
    assert!(
        err.retry_after_secs > 0,
        "retry_after_secs should be positive"
    );
}

#[test]
fn test_rate_limiter_per_tool_isolation() {
    let limiter = RateLimiter::new(60, 1);

    assert!(limiter.check("tool-a").is_ok());
    assert!(limiter.check("tool-a").is_err(), "Second call to same tool should be rate limited");
    assert!(limiter.check("tool-b").is_ok(), "Different tool has independent counter");
}

#[test]
fn test_rate_limiter_concurrent_safety() {
    let limiter = Arc::new(RateLimiter::new(60, 100));
    let mut handles = Vec::new();

    for i in 0..10 {
        let l = limiter.clone();
        handles.push(std::thread::spawn(move || {
            for _ in 0..10 {
                let _ = l.check(&format!("thread-{}", i));
            }
        }));
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }
    // No panic means concurrent access is safe
}

// ============================================================================
// Token Store
// ============================================================================

#[test]
fn test_token_store_missing_token_returns_none() {
    let db = temp_db("missing");
    let _ = std::fs::remove_file(&db);
    let key: [u8; 32] = rand::random();
    let store = TokenStore::new(db, &key).expect("Create token store");
    assert!(store.load_token("unknown").unwrap().is_none());
}

#[test]
fn test_token_store_save_load_roundtrip() {
    let db = temp_db("roundtrip");
    let _ = std::fs::remove_file(&db);
    let key: [u8; 32] = rand::random();
    let store = TokenStore::new(db, &key).expect("Create token store");

    store
        .save_token("my-app", "super-secret-token", 9_999_999_999)
        .expect("Save token");

    let (token, expires) = store
        .load_token("my-app")
        .expect("Load token")
        .expect("Token should exist");

    assert_eq!(token, "super-secret-token");
    assert_eq!(expires, 9_999_999_999);
}

#[test]
fn test_token_store_overwrite_existing() {
    let db = temp_db("overwrite");
    let _ = std::fs::remove_file(&db);
    let key: [u8; 32] = rand::random();
    let store = TokenStore::new(db, &key).expect("Create token store");

    store.save_token("my-app", "old", 9_999_999_999).unwrap();
    store.save_token("my-app", "new", 9_999_999_999).unwrap();

    let (token, _) = store.load_token("my-app").unwrap().unwrap();
    assert_eq!(token, "new", "Save should overwrite existing token");
}

// ============================================================================
// FeishuClient
// ============================================================================

#[test]
fn test_client_construction_and_accessors() {
    let db = temp_db("client");
    let _ = std::fs::remove_file(&db);
    let key: [u8; 32] = rand::random();
    let store = TokenStore::new(db, &key).expect("Create token store");

    let client = FeishuClient::new(
        "https://example.com/api".to_string(),
        Arc::new(store),
        "my-app".to_string(),
    );

    assert_eq!(client.base_url(), "https://example.com/api");
    assert_eq!(client.app_id(), "my-app");
}

// ============================================================================
// FeishuMcpServer
// ============================================================================

#[test]
fn test_server_constructs() {
    // Construction should succeed without panicking
    let _server = create_test_server();
}

#[test]
fn test_server_is_send_sync() {
    // Verifies FeishuMcpServer implements Send + Sync
    // by sharing an Arc across threads
    let server = Arc::new(create_test_server());
    let cloned = server.clone();

    let handle = std::thread::spawn(move || {
        let _ = cloned.rate_limiter.check("test");
    });

    handle.join().expect("Thread should complete");
    let _ = server.rate_limiter.check("test");
}
