// mlb-stats-api/tests/client.rs
//
// HTTP-level tests for MlbClient using wiremock.
//
// These tests verify that each client method:
//   1. Constructs the correct URL and query parameters
//   2. Handles HTTP error responses correctly (4xx, 5xx)
//   3. Handles malformed responses gracefully
//
// They do NOT test model deserialization correctness — that is covered by
// deserialization.rs using real fixture files. They do NOT test consumer
// logic — that is covered by mock_client.rs using MockMlbClient.
//
// # Setup
//
// Each test calls `setup()` which starts a fresh wiremock server and returns
// an MlbClient pointed at it. Both `base_url` and `live_base_url` point at
// the same wiremock server — the path matching keeps them distinct.
//
// wiremock mocks are scoped to the MockServer instance lifetime. When the
// server drops at the end of each test, all mocks are cleared automatically.

use mlb_stats_api::{MlbApiError, MlbClient};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Minimal valid JSON stubs
//
// These are the smallest payloads that will deserialize into each response
// type. They are not real API responses — use deserialization.rs fixtures
// for that. The goal here is just to get past deserialization so we can
// assert on the HTTP behavior.
// ---------------------------------------------------------------------------

const EMPTY_SCHEDULE: &str =
    r#"{"copyright":"","totalItems":0,"totalEvents":0,"totalGames":0,"totalGamesInProgress":0,"dates":[]}"#;

const EMPTY_STANDINGS: &str = r#"{"records":[]}"#;

const MINIMAL_LIVE_FEED: &str =
    r#"{"copyright":"","gamePk":825024,"gameData":null,"liveData":null}"#;

const EMPTY_ARRAY: &str = r#"[]"#;

const EMPTY_OBJECT: &str = r#"{}"#;

const EMPTY_TEAMS: &str = r#"{"teams":[]}"#;

const EMPTY_ROSTER: &str = r#"{"roster":[]}"#;

const EMPTY_PEOPLE: &str = r#"{"people":[]}"#;

const EMPTY_VENUES: &str = r#"{"venues":[]}"#;

const EMPTY_LEAGUES: &str = r#"{"leagues":[]}"#;

const EMPTY_DIVISIONS: &str = r#"{"divisions":[]}"#;

const EMPTY_CONFERENCES: &str = r#"{"conferences":[]}"#;

const EMPTY_SPORTS: &str = r#"{"sports":[]}"#;

const EMPTY_SEASONS: &str = r#"{"seasons":[]}"#;

const EMPTY_STATS: &str = r#"{"stats":[]}"#;

const EMPTY_LEADERS: &str = r#"{"leaderCategories":[]}"#;

const EMPTY_TEAM_LEADERS: &str = r#"{"teamLeaders":[]}"#;

const EMPTY_ATTENDANCE: &str = r#"{"records":null,"aggregates":[]}"#;

const EMPTY_COACHES: &str = r#"{"roster":[]}"#;

const EMPTY_UMPIRES: &str = r#"{"sports":[]}"#;

const EMPTY_UMPIRE_SCHEDULE: &str = r#"{"roster":[]}"#;

const MINIMAL_DIFF_PATCH: &str = r#"{"gamePk":825024,"gameData":null,"liveData":null}"#;

const MINIMAL_TIMESTAMPS: &str = r#"{"gameData":null}"#;

const MINIMAL_LINESCORE: &str = r#"{"currentInning":null}"#;

const MINIMAL_BOXSCORE: &str = r#"{"teams":null}"#;

const MINIMAL_PLAY_BY_PLAY: &str = r#"{"allPlays":[]}"#;

const MINIMAL_CONTENT: &str = r#"{"link":null}"#;

const MINIMAL_CONTEXT_METRICS: &str = r#"{"gamePk":null}"#;

const MINIMAL_GAME_CHANGES: &str = r#"{"gamePk":null}"#;

const MINIMAL_GAME_PACE: &str = r#"{"seasons":[]}"#;

const MINIMAL_HIGH_LOW: &str = r#"{"results":[]}"#;

// ---------------------------------------------------------------------------
// Test setup
// ---------------------------------------------------------------------------

struct TestEnv {
    server: MockServer,
    client: MlbClient,
}

async fn setup() -> TestEnv {
    let server = MockServer::start().await;
    let url = server.uri();
    let client = MlbClient::new()
        .with_base_url(&format!("{url}/api/v1"))
        .with_live_base_url(&format!("{url}/api/v1.1"));
    TestEnv { server, client }
}

// ---------------------------------------------------------------------------
// Schedule
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_schedule_today_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/schedule"))
        .and(query_param("sportId", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_SCHEDULE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_schedule_today().await;
    assert!(result.is_ok(), "get_schedule_today failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_schedule_for_date_includes_team_and_date() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/schedule"))
        .and(query_param("sportId", "1"))
        .and(query_param("teamId", "140"))
        // The client converts YYYY-MM-DD → MM/DD/YYYY internally
        .and(query_param("date", "04/15/2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_SCHEDULE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_schedule_for_date(140u32, "2026-04-15").await;
    assert!(result.is_ok(), "get_schedule_for_date failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Live game feed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_live_game_uses_v1_1_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1.1/game/825024/feed/live"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_LIVE_FEED))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_live_game(825024).await;
    assert!(result.is_ok(), "get_live_game failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_live_game_fields_appends_fields_param() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1.1/game/825024/feed/live"))
        .and(query_param("fields", "gamePk,gameData"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_LIVE_FEED))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_live_game_fields(825024, "gamePk,gameData").await;
    assert!(result.is_ok(), "get_live_game_fields failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_live_game_diff_patch_includes_start_time_code() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1.1/game/825024/feed/live/diffPatch"))
        .and(query_param("startTimeCode", "20260415_190000"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_DIFF_PATCH))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env
        .client
        .get_live_game_diff_patch(825024, "20260415_190000")
        .await;
    assert!(result.is_ok(), "get_live_game_diff_patch failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_live_game_timestamps_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1.1/game/825024/feed/live/timestamps"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_TIMESTAMPS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_live_game_timestamps(825024).await;
    assert!(result.is_ok(), "get_live_game_timestamps failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Standings
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_standings_includes_league_and_season() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/standings"))
        .and(query_param("leagueId", "103"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_STANDINGS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_standings(103u32, 2026).await;
    assert!(result.is_ok(), "get_standings failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Teams & Roster
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_teams_includes_sport_and_season() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/teams"))
        .and(query_param("sportId", "1"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_TEAMS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_teams(2026).await;
    assert!(result.is_ok(), "get_teams failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_roster_includes_team_type_season() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/teams/140/roster"))
        .and(query_param("rosterType", "active"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_ROSTER))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_roster(140u32, "active", 2026).await;
    assert!(result.is_ok(), "get_roster failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// People
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_person_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/people/545361"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PEOPLE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_person(545361).await;
    assert!(result.is_ok(), "get_person failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Standalone game endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_linescore_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/game/825024/linescore"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_LINESCORE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_linescore(825024).await;
    assert!(result.is_ok(), "get_linescore failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_boxscore_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/game/825024/boxscore"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_BOXSCORE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_boxscore(825024).await;
    assert!(result.is_ok(), "get_boxscore failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_play_by_play_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/game/825024/playByPlay"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_PLAY_BY_PLAY))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_play_by_play(825024).await;
    assert!(result.is_ok(), "get_play_by_play failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_win_probability_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/game/825024/winProbability"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_ARRAY))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_win_probability(825024).await;
    assert!(result.is_ok(), "get_win_probability failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_game_content_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/game/825024/content"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_CONTENT))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_game_content(825024).await;
    assert!(result.is_ok(), "get_game_content failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_context_metrics_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/game/825024/contextMetrics"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_CONTEXT_METRICS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_context_metrics(825024).await;
    assert!(result.is_ok(), "get_context_metrics failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_game_changes_includes_updated_since() {
    let env = setup().await;
    // swagger: GET /v1/game/changes — bulk feed, gamePk is not a parameter
    Mock::given(method("GET"))
        .and(path("/api/v1/game/changes"))
        .and(query_param("updatedSince", "2026-04-15T19:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_GAME_CHANGES))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env
        .client
        .get_game_changes("2026-04-15T19:00:00Z")
        .await;
    assert!(result.is_ok(), "get_game_changes failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_game_pace_includes_season() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/gamePace"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MINIMAL_GAME_PACE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_game_pace(2026).await;
    assert!(result.is_ok(), "get_game_pace failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Venue / League / Division / Sport
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_venues_includes_venue_id() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/venues"))
        .and(query_param("venueId", "5325"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_VENUES))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_venues(5325).await;
    assert!(result.is_ok(), "get_venues failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_leagues_includes_sport_id() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/league"))
        .and(query_param("sportId", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_LEAGUES))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_leagues().await;
    assert!(result.is_ok(), "get_leagues failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_divisions_includes_sport_id() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/divisions"))
        .and(query_param("sportId", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_DIVISIONS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_divisions().await;
    assert!(result.is_ok(), "get_divisions failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_conferences_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/conferences"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_CONFERENCES))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_conferences().await;
    assert!(result.is_ok(), "get_conferences failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_sports_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/sports"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_SPORTS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_sports().await;
    assert!(result.is_ok(), "get_sports failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Season
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_season_includes_season_and_sport() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/seasons/2026"))
        .and(query_param("sportId", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_SEASONS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_season(2026, 1).await;
    assert!(result.is_ok(), "get_season failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_all_seasons_includes_all_flag() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/seasons"))
        .and(query_param("sportId", "1"))
        .and(query_param("all", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_SEASONS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_all_seasons(1).await;
    assert!(result.is_ok(), "get_all_seasons failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_league_leaders_includes_category_and_season() {
    use mlb_stats_api::client::LeaderParams;

    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/stats/leaders"))
        .and(query_param("leaderCategories", "homeRuns"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_LEADERS))
        .expect(1)
        .mount(&env.server)
        .await;

    let params = LeaderParams::new("homeRuns").season(2026);
    let result = env.client.get_league_leaders(&params).await;
    assert!(result.is_ok(), "get_league_leaders failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_team_leaders_includes_team_id() {
    use mlb_stats_api::client::LeaderParams;

    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/teams/140/leaders"))
        .and(query_param("leaderCategories", "homeRuns"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_TEAM_LEADERS))
        .expect(1)
        .mount(&env.server)
        .await;

    let params = LeaderParams::new("homeRuns");
    let result = env.client.get_team_leaders(140u32, &params).await;
    assert!(result.is_ok(), "get_team_leaders failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Attendance / Coaches / Umpires
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_attendance_includes_team_and_season() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/attendance"))
        .and(query_param("teamId", "140"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_ATTENDANCE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_attendance(140u32, 2026).await;
    assert!(result.is_ok(), "get_attendance failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_coaches_includes_team_and_season() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/teams/140/coaches"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_COACHES))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_coaches(140u32, 2026).await;
    assert!(result.is_ok(), "get_coaches failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_umpires_hits_correct_path() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/jobs/umpires"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_UMPIRES))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_umpires().await;
    assert!(result.is_ok(), "get_umpires failed: {:?}", result.err());
}

#[tokio::test]
async fn client_get_umpire_schedule_includes_umpire_and_season() {
    let env = setup().await;
    // swagger: GET /v1/jobs/umpires/games/{umpireId} — umpireId is a path param
    Mock::given(method("GET"))
        .and(path("/api/v1/jobs/umpires/games/427119"))
        .and(query_param("season", "2026"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_UMPIRE_SCHEDULE))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_umpire_schedule(427119, 2026).await;
    assert!(result.is_ok(), "get_umpire_schedule failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_meta_includes_type_param() {
    use mlb_stats_api::MetaType;

    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/meta"))
        .and(query_param("type", "gameTypes"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_ARRAY))
        .expect(1)
        .mount(&env.server)
        .await;

    let result = env.client.get_meta(MetaType::GameTypes).await;
    assert!(result.is_ok(), "get_meta(GameTypes) failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// Raw escape hatch
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_get_raw_passes_url_through() {
    let env = setup().await;
    let url = format!("{}/api/v1/sports", env.server.uri());

    Mock::given(method("GET"))
        .and(path("/api/v1/sports"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_SPORTS))
        .expect(1)
        .mount(&env.server)
        .await;

    let result: Result<serde_json::Value, _> = env.client.get(&url).await;
    assert!(result.is_ok(), "raw get failed: {:?}", result.err());
}

// ---------------------------------------------------------------------------
// HTTP error handling
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_returns_rate_limited_on_429() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/schedule"))
        .respond_with(
            ResponseTemplate::new(429).insert_header("Retry-After", "60"),
        )
        .mount(&env.server)
        .await;

    let result = env.client.get_schedule_today().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        MlbApiError::RateLimited { retry_after_secs } => {
            assert_eq!(retry_after_secs, 60, "should parse Retry-After header");
        }
        other => panic!("expected RateLimited, got {other:?}"),
    }
}

#[tokio::test]
async fn client_returns_unexpected_response_on_500() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/schedule"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&env.server)
        .await;

    let result = env.client.get_schedule_today().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        MlbApiError::Http(_) | MlbApiError::UnexpectedResponse(_)
    ));
}

#[tokio::test]
async fn client_returns_deserialize_error_on_malformed_json() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/schedule"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("<html>not json</html>")
                .insert_header("Content-Type", "text/html"),
        )
        .mount(&env.server)
        .await;

    let result = env.client.get_schedule_today().await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        MlbApiError::Deserialize(_)
    ));
}

#[tokio::test]
async fn client_returns_error_on_404() {
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/game/999999999/linescore"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"message":"Game not found"}"#))
        .mount(&env.server)
        .await;

    let result = env.client.get_linescore(999_999_999).await;
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// wiremock expectation verification
//
// wiremock asserts `.expect(N)` counts when the MockServer drops (end of
// each test). If a mock's expected call count is not satisfied, the test
// panics. This gives us a second layer of verification beyond just checking
// the Result — it confirms the client actually made the request rather than
// returning a cached or default response.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn client_does_not_make_extra_requests() {
    // Mount a mock that only allows exactly 1 call.
    // If the client made 0 or 2+ calls, wiremock panics on drop.
    let env = setup().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/schedule"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_SCHEDULE))
        .expect(1) // exactly one call
        .mount(&env.server)
        .await;

    env.client.get_schedule_today().await.unwrap();
    // MockServer drops here — wiremock verifies the expect(1) was met
}