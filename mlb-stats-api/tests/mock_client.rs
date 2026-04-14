// mlb-stats-api/tests/mock_client.rs
//
// Tests for MockMlbClient.
//
// These tests verify that the mock client:
//   1. Serves fixture JSON correctly
//   2. Records calls faithfully  
//   3. Injects errors when configured
//   4. Returns a clear error when no fixture is loaded for a route
//
// These tests do NOT require network access.

#![cfg(feature = "test-utils")]

use mlb_stats_api::mock::{MockError, MockMlbClient};

// Minimal valid JSON for each response type used in these tests.
// In real consumer tests, use include_str! with actual fixture files.
const MINIMAL_SCHEDULE: &str = r#"{
    "copyright": "",
    "totalItems": 1,
    "totalEvents": 0,
    "totalGames": 1,
    "totalGamesInProgress": 0,
    "dates": []
}"#;

const MINIMAL_STANDINGS: &str = r#"{
    "records": []
}"#;

const MINIMAL_LIVE_FEED: &str = r#"{
    "copyright": "",
    "gamePk": 825024,
    "gameData": null,
    "liveData": null
}"#;

const MINIMAL_ROSTER: &str = r#"{
    "copyright": "",
    "roster": [],
    "teamId": 147,
    "rosterType": "active"
}"#;

// ---------------------------------------------------------------------------
// Basic fixture serving
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mock_serves_schedule_fixture() {
    let client = MockMlbClient::new().with_response("schedule", MINIMAL_SCHEDULE);
    let result = client.get_schedule_today().await;
    assert!(result.is_ok(), "should serve schedule fixture: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.total_items, Some(1));
}

#[tokio::test]
async fn mock_serves_standings_fixture() {
    let client = MockMlbClient::new().with_response("standings", MINIMAL_STANDINGS);
    let result = client.get_standings(103u32, 2023).await;
    assert!(result.is_ok(), "should serve standings fixture: {:?}", result.err());
}

#[tokio::test]
async fn mock_serves_live_game_fixture() {
    let client = MockMlbClient::new().with_response("live_game", MINIMAL_LIVE_FEED);
    let result = client.get_live_game(825024).await;
    assert!(result.is_ok(), "should serve live_game fixture: {:?}", result.err());
    let feed = result.unwrap();
    assert_eq!(feed.game_pk, Some(825024));
}

// ---------------------------------------------------------------------------
// Call recording
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mock_records_calls() {
    let client = MockMlbClient::new()
        .with_response("schedule", MINIMAL_SCHEDULE)
        .with_response("standings", MINIMAL_STANDINGS);

    client.get_schedule_today().await.unwrap();
    client.get_standings(103u32, 2023).await.unwrap();

    let calls = client.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].route, "schedule");
    assert_eq!(calls[1].route, "standings");
}

#[tokio::test]
async fn mock_was_called() {
    let client = MockMlbClient::new().with_response("schedule", MINIMAL_SCHEDULE);
    assert!(!client.was_called("schedule"), "should not be called yet");
    client.get_schedule_today().await.unwrap();
    assert!(client.was_called("schedule"), "should be called after invocation");
    assert!(!client.was_called("standings"), "standings should not have been called");
}

#[tokio::test]
async fn mock_call_count() {
    let client = MockMlbClient::new().with_response("schedule", MINIMAL_SCHEDULE);
    assert_eq!(client.call_count("schedule"), 0);
    client.get_schedule_today().await.unwrap();
    client.get_schedule_today().await.unwrap();
    assert_eq!(client.call_count("schedule"), 2);
}

#[tokio::test]
async fn mock_records_params() {
    let client = MockMlbClient::new().with_response("roster", MINIMAL_ROSTER);
    client.get_roster(147u32, "active", 2023).await.unwrap();
    let calls = client.calls();
    assert_eq!(calls[0].route, "roster");
    assert!(
        calls[0].params.iter().any(|p| p.contains("147")),
        "call params should contain teamId=147"
    );
    assert!(
        calls[0].params.iter().any(|p| p.contains("active")),
        "call params should contain rosterType=active"
    );
}

#[tokio::test]
async fn mock_reset_calls() {
    let client = MockMlbClient::new().with_response("schedule", MINIMAL_SCHEDULE);
    client.get_schedule_today().await.unwrap();
    assert_eq!(client.call_count("schedule"), 1);
    client.reset_calls();
    assert_eq!(client.call_count("schedule"), 0, "call count should be 0 after reset");
}

// ---------------------------------------------------------------------------
// Missing fixture error
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mock_missing_fixture_returns_error() {
    let client = MockMlbClient::new(); // no fixtures loaded
    let result = client.get_schedule_today().await;
    assert!(result.is_err(), "should error when no fixture is loaded for route");
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("schedule"),
        "error message should mention the route key, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// Error injection
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mock_error_override_rate_limited() {
    let client = MockMlbClient::new()
        .with_response("schedule", MINIMAL_SCHEDULE)
        .with_error(MockError::RateLimited { retry_after_secs: 60 });
    let result = client.get_schedule_today().await;
    assert!(result.is_err());
    // Call should still be recorded even when error override is active
    assert_eq!(client.call_count("schedule"), 1);
    match result.unwrap_err() {
        mlb_stats_api::error::MlbApiError::RateLimited { retry_after_secs } => {
            assert_eq!(retry_after_secs, 60);
        }
        other => panic!("expected RateLimited, got {other:?}"),
    }
}

#[tokio::test]
async fn mock_error_override_network_unavailable() {
    let client = MockMlbClient::new()
        .with_error(MockError::NetworkUnavailable);
    let result = client.get_live_game(825024).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        mlb_stats_api::error::MlbApiError::NetworkUnavailable
    ));
}

#[tokio::test]
async fn mock_error_override_deserialize() {
    let client = MockMlbClient::new()
        .with_error(MockError::Deserialize);
    let result = client.get_schedule_today().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        mlb_stats_api::error::MlbApiError::Deserialize(_)
    ));
}

// ---------------------------------------------------------------------------
// Clone / shared-state behavior
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mock_cloned_shares_calls() {
    // MockMlbClient uses Arc<Mutex<>> for calls, so clones share call state.
    // This mirrors how you might pass a mock into multiple async tasks.
    let client = MockMlbClient::new().with_response("schedule", MINIMAL_SCHEDULE);
    let client2 = client.clone();
    client.get_schedule_today().await.unwrap();
    // Both handles see the same calls
    assert_eq!(
        client2.call_count("schedule"),
        1,
        "cloned client should share call state"
    );
}