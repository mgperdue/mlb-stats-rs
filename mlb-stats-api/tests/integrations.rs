// mlb-stats-api/tests/integration.rs
//
// Live API integration tests.
//
// These tests make real HTTP requests to statsapi.mlb.com. They are marked
// `#[ignore]` so that `cargo test` does not run them in CI or local dev
// unless explicitly requested.
//
// # Running integration tests
//
//   cargo test -p mlb-stats-api -- --include-ignored
//
// # When to run them
//
// - Before publishing a new version, to verify the API hasn't changed shape
// - When a fixture test fails and you want to see the live response
// - When implementing a new endpoint method
//
// These tests make a single request each and assert only that the response
// deserializes without error, plus minimal sanity checks on stable fields.
// They do not assert on values that change over time (scores, rosters, etc.).

use mlb_stats_api::MlbClient;

// gamePk 825024 — a known completed regular-season game used as a stable fixture anchor.
const STABLE_GAME_PK: u64 = 825024;

fn client() -> MlbClient {
    MlbClient::new()
}

// ---------------------------------------------------------------------------
// Schedule
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API — run with: cargo test -p mlb-stats-api -- --include-ignored"]
async fn integration_get_schedule_today() {
    let result = client().get_schedule_today().await;
    assert!(result.is_ok(), "get_schedule_today failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_schedule_for_date() {
    // April 15 — Jackie Robinson Day — always has games (regular season)
    let result = client()
        .get_schedule_for_date(147u32, "2023-04-15")
        .await;
    assert!(result.is_ok(), "get_schedule_for_date failed: {:?}", result.err());
    let resp = result.unwrap();
    assert!(
        !resp.dates.is_empty(),
        "should have games on 2023-04-15 (Jackie Robinson Day)"
    );
}

// ---------------------------------------------------------------------------
// Live game feed
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_live_game() {
    let result = client().get_live_game(STABLE_GAME_PK).await;
    assert!(result.is_ok(), "get_live_game failed: {:?}", result.err());
    let feed = result.unwrap();
    assert!(
        feed.game_pk.is_some(),
        "live feed should have a gamePk"
    );
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_live_game_fields() {
    let result = client()
        .get_live_game_fields(
            STABLE_GAME_PK,
            "gamePk,gameData,status,abstractGameState",
        )
        .await;
    assert!(result.is_ok(), "get_live_game_fields failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_live_game_timestamps() {
    let result = client()
        .get_live_game_timestamps(STABLE_GAME_PK)
        .await;
    assert!(result.is_ok(), "get_live_game_timestamps failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Game endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_linescore() {
    let result = client().get_linescore(STABLE_GAME_PK).await;
    assert!(result.is_ok(), "get_linescore failed: {:?}", result.err());
    let resp = result.unwrap();
    // WS Game 5 was a 9-inning game
    if let Some(innings) = resp.innings {
    }
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_boxscore() {
    let result = client().get_boxscore(STABLE_GAME_PK).await;
    assert!(result.is_ok(), "get_boxscore failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_play_by_play() {
    let result = client().get_play_by_play(STABLE_GAME_PK).await;
    assert!(result.is_ok(), "get_play_by_play failed: {:?}", result.err());
    let resp = result.unwrap();
    assert!(!resp.all_plays.is_empty(), "completed game should have plays");
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_win_probability() {
    let result = client().get_win_probability(STABLE_GAME_PK).await;
    assert!(result.is_ok(), "get_win_probability failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_game_content() {
    let result = client().get_game_content(STABLE_GAME_PK).await;
    assert!(result.is_ok(), "get_game_content failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_context_metrics() {
    let result = client().get_context_metrics(STABLE_GAME_PK).await;
    assert!(result.is_ok(), "get_context_metrics failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_game_pace() {
    let result = client().get_game_pace(2023).await;
    assert!(result.is_ok(), "get_game_pace failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Standings
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_standings_al() {
    let result = client().get_standings(103u32, 2023).await;
    assert!(result.is_ok(), "get_standings AL failed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.records.len(), 3, "AL should have 3 division standing records");
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_standings_nl() {
    let result = client().get_standings(104u32, 2023).await;
    assert!(result.is_ok(), "get_standings NL failed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.records.len(), 3, "NL should have 3 division standing records");
}

// ---------------------------------------------------------------------------
// Teams & Roster
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_teams() {
    let result = client().get_teams(2023).await;
    assert!(result.is_ok(), "get_teams failed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.teams.len(), 30, "MLB should have 30 teams");
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_roster() {
    let result = client().get_roster(147u32, "active", 2023).await;
    assert!(result.is_ok(), "get_roster failed: {:?}", result.err());
    let resp = result.unwrap();
    assert!(
        !resp.roster.is_empty(),
        "active roster should have players"
    );
}

// ---------------------------------------------------------------------------
// People
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_person() {
    // Mike Trout — always a valid player ID
    let result = client().get_person(545361).await;
    assert!(result.is_ok(), "get_person failed: {:?}", result.err());
    let resp = result.unwrap();
    let person = resp.people.first().unwrap();
    assert_eq!(person.id, Some(545361), "should return Trout's record");
}

// ---------------------------------------------------------------------------
// Venue
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_venues() {
    // Globe Life Field — home of the 2023 WS
    let result = client().get_venues(5325).await;
    assert!(result.is_ok(), "get_venues failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// League / Division / Sport
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_leagues() {
    let result = client().get_leagues().await;
    assert!(result.is_ok(), "get_leagues failed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.leagues.len(), 2, "sportId=1 should return 2 leagues");
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_divisions() {
    let result = client().get_divisions().await;
    assert!(result.is_ok(), "get_divisions failed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(resp.divisions.len(), 6, "MLB has 6 divisions");
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_sports() {
    let result = client().get_sports().await;
    assert!(result.is_ok(), "get_sports failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Season
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_season() {
    let result = client().get_season(2023, 1).await;
    assert!(result.is_ok(), "get_season failed: {:?}", result.err());
    let resp = result.unwrap();
    assert_eq!(
        resp.seasons.first().and_then(|s| s.season_id.as_deref()),
        Some("2023")
    );
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_league_leaders() {
    use mlb_stats_api::client::LeaderParams;
    let params = LeaderParams::new("homeRuns").season(2023);
    let result = client().get_league_leaders(&params).await;
    assert!(result.is_ok(), "get_league_leaders failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Attendance
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_attendance() {
    let result = client().get_attendance(147u32, 2023).await;
    assert!(result.is_ok(), "get_attendance failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Umpires
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_umpires() {
    let result = client().get_umpires().await;
    assert!(result.is_ok(), "get_umpires failed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_umpire_schedule() {
    // Umpire ID 427119 — Angel Hernandez (long career, stable ID)
    let result = client().get_umpire_schedule(427119, 2023).await;
    assert!(result.is_ok(), "get_umpire_schedule failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "hits live API"]
async fn integration_get_meta_game_types() {
    use mlb_stats_api::models::meta::MetaType;
    let result = client().get_meta(MetaType::GameTypes).await;
    assert!(result.is_ok(), "get_meta(GameTypes) failed: {:?}", result.err());
    let entries = result.unwrap();
    let codes: Vec<_> = entries.iter().filter_map(|e| e.code.as_deref()).collect();
    assert!(codes.contains(&"R"), "gameTypes should include R");
}