// mlb-stats-api/src/mock.rs
//
// Available when the `test-utils` feature is enabled.
//
// `MockMlbClient` is a drop-in replacement for `MlbClient` in test code.
// It serves pre-loaded JSON responses without making any network requests.
//
// # Design
//
// The mock is keyed by a string "route key" — a short canonical identifier
// for each endpoint (e.g., `"schedule"`, `"live_game"`, `"standings"`).
// Consumers load fixture JSON, set it on the mock, and then call the same
// methods they would call on a real `MlbClient`.
//
// The mock records every call made so tests can assert on call order,
// call count, and parameters.
//
// # Usage
//
// ```rust
// use mlb_stats_api::mock::MockMlbClient;
//
// let fixture = include_str!("../tests/fixtures/schedule/for_date.json");
//
// let client = MockMlbClient::new()
//     .with_response("schedule", fixture);
//
// let schedule = client.get_schedule_for_date(147, "2024-04-15").await.unwrap();
// assert!(client.was_called("schedule"));
// ```

#![cfg(feature = "test-utils")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::MlbApiError;
use crate::models::{
    attendance::AttendanceResponse,
    common::{LeagueId, TeamId},
    game::{
        BoxscoreResponse, ContextMetrics, GameChangesResponse, GameContent, GamePaceResponse,
        HighLowResponse, LinescoreResponse, WinProbabilityEntry,
    },
    league::{ConferencesResponse, DivisionsResponse, LeagueResponse, SportsResponse},
    live::LiveGameFeed,
    meta::{MetaEntry, MetaType},
    plays::{DiffPatchResponse, PlayByPlayResponse, TimestampsResponse},
    roster::{PeopleResponse, RosterResponse, TeamsResponse},
    schedule::ScheduleResponse,
    season::SeasonsResponse,
    standings::StandingsResponse,
    stats::{LeagueLeadersResponse, StatsResponse, TeamLeadersResponse},
    umpires::{CoachesResponse, UmpireScheduleResponse, UmpiresResponse},
    venue::VenuesResponse,
};

/// A record of a single call made to the mock client.
#[derive(Debug, Clone)]
pub struct MockCall {
    /// The route key for the endpoint (e.g. `"schedule"`, `"live_game"`).
    pub route: String,
    /// Any parameters passed as part of the call, serialized to strings.
    /// The format is `"name=value"` pairs.
    pub params: Vec<String>,
}

/// A fixture-backed mock of [`MlbClient`][crate::MlbClient] for use in tests.
///
/// Load JSON fixtures with [`with_response`][MockMlbClient::with_response],
/// then call the same async methods you would call on the real client.
/// Every call is recorded and inspectable via [`calls`][MockMlbClient::calls].
///
/// # Route Keys
///
/// Each endpoint method is associated with a stable route key string.
/// The route key is what you pass to `with_response` and what appears in
/// recorded calls. A table of all route keys is in the crate-level
/// `test-utils` documentation.
///
/// | Method | Route key |
/// |--------|-----------|
/// | `get_schedule_today` | `"schedule"` |
/// | `get_schedule_for_date` | `"schedule"` |
/// | `get_schedule` | `"schedule"` |
/// | `get_live_game` | `"live_game"` |
/// | `get_live_game_fields` | `"live_game"` |
/// | `get_live_game_diff_patch` | `"live_game_diff_patch"` |
/// | `get_live_game_timestamps` | `"live_game_timestamps"` |
/// | `get_standings` | `"standings"` |
/// | `get_teams` | `"teams"` |
/// | `get_roster` | `"roster"` |
/// | `get_person` | `"person"` |
/// | `get_linescore` | `"linescore"` |
/// | `get_boxscore` | `"boxscore"` |
/// | `get_play_by_play` | `"play_by_play"` |
/// | `get_win_probability` | `"win_probability"` |
/// | `get_game_content` | `"game_content"` |
/// | `get_context_metrics` | `"context_metrics"` |
/// | `get_game_changes` | `"game_changes"` | bulk feed, no gamePk |
/// | `get_game_pace` | `"game_pace"` |
/// | `get_high_low` | `"high_low"` |
/// | `get_venues` | `"venues"` |
/// | `get_leagues` | `"leagues"` |
/// | `get_divisions` | `"divisions"` |
/// | `get_conferences` | `"conferences"` |
/// | `get_sports` | `"sports"` |
/// | `get_season` | `"season"` |
/// | `get_all_seasons` | `"seasons"` |
/// | `get_stats` | `"stats"` |
/// | `get_team_stats` | `"team_stats"` |
/// | `get_league_leaders` | `"league_leaders"` |
/// | `get_team_leaders` | `"team_leaders"` |
/// | `get_stat_streaks` | `"stat_streaks"` |
/// | `get_stats_metrics` | `"stats_metrics"` |
/// | `get_attendance` | `"attendance"` |
/// | `get_coaches` | `"coaches"` |
/// | `get_umpires` | `"umpires"` |
/// | `get_umpire_schedule` | `"umpire_schedule"` |
/// | `get_meta` | `"meta"` |
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "test-utils")]
/// # async fn example() {
/// use mlb_stats_api::mock::MockMlbClient;
///
/// let fixture = include_str!("../tests/fixtures/schedule/for_date.json");
/// let client = MockMlbClient::new().with_response("schedule", fixture);
///
/// let result = client.get_schedule_for_date(147u32, "2024-04-15").await;
/// assert!(result.is_ok());
/// assert_eq!(client.call_count("schedule"), 1);
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct MockMlbClient {
    responses: Arc<HashMap<String, String>>,
    calls: Arc<Mutex<Vec<MockCall>>>,
    /// If set, every call returns this error instead of a fixture.
    error_override: Option<MockError>,
}

/// The error that `MockMlbClient` can be configured to return.
#[derive(Debug, Clone)]
pub enum MockError {
    /// Simulate a deserialization failure.
    Deserialize,
    /// Simulate a 429 / rate-limited response.
    RateLimited { retry_after_secs: u64 },
    /// Simulate a generic unexpected response.
    UnexpectedResponse(String),
    /// Simulate network unavailability.
    NetworkUnavailable,
}

impl MockMlbClient {
    /// Create a new mock client with no pre-loaded responses.
    ///
    /// Calling any endpoint method before loading the corresponding fixture
    /// returns [`MlbApiError::UnexpectedResponse`] with a message indicating
    /// the missing route key.
    pub fn new() -> Self {
        MockMlbClient {
            responses: Arc::new(HashMap::new()),
            calls: Arc::new(Mutex::new(Vec::new())),
            error_override: None,
        }
    }

    /// Load a JSON fixture for a route key.
    ///
    /// `json` should be the verbatim content of a fixture file —
    /// typically loaded with [`include_str!`] at compile time or
    /// [`std::fs::read_to_string`] at runtime.
    ///
    /// Multiple routes can be loaded by chaining calls:
    ///
    /// ```rust
    /// # #[cfg(feature = "test-utils")]
    /// # {
    /// use mlb_stats_api::mock::MockMlbClient;
    /// let client = MockMlbClient::new()
    ///     .with_response("schedule", r#"{"dates":[],"totalItems":0,"totalEvents":0,"totalGames":0,"totalGamesInProgress":0}"#)
    ///     .with_response("standings", r#"{"records":[]}"#);
    /// # }
    /// ```
    #[must_use]
    pub fn with_response(mut self, route: impl Into<String>, json: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.responses).insert(route.into(), json.into());
        self
    }

    /// Configure the mock to return an error for every call, regardless of
    /// whether fixtures are loaded.
    ///
    /// Useful for testing error-handling paths in consumers.
    #[must_use]
    pub fn with_error(mut self, err: MockError) -> Self {
        self.error_override = Some(err);
        self
    }

    /// Return all recorded calls in order.
    pub fn calls(&self) -> Vec<MockCall> {
        self.calls.lock().unwrap().clone()
    }

    /// Return true if the given route was called at least once.
    pub fn was_called(&self, route: &str) -> bool {
        self.calls.lock().unwrap().iter().any(|c| c.route == route)
    }

    /// Return the number of times the given route was called.
    pub fn call_count(&self, route: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|c| c.route == route)
            .count()
    }

    /// Reset recorded calls without changing loaded fixtures.
    pub fn reset_calls(&self) {
        self.calls.lock().unwrap().clear();
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn record(&self, route: &str, params: Vec<String>) {
        self.calls.lock().unwrap().push(MockCall {
            route: route.to_owned(),
            params,
        });
    }

    fn resolve<T: serde::de::DeserializeOwned>(
        &self,
        route: &str,
        params: Vec<String>,
    ) -> Result<T, MlbApiError> {
        self.record(route, params);

        if let Some(ref err) = self.error_override {
            return Err(match err {
                MockError::Deserialize => {
                    // Produce a real serde_json error by trying to parse invalid JSON.
                    serde_json::from_str::<serde_json::Value>("not json")
                        .unwrap_err()
                        .into()
                }
                MockError::RateLimited { retry_after_secs } => MlbApiError::RateLimited {
                    retry_after_secs: *retry_after_secs,
                },
                MockError::UnexpectedResponse(msg) => {
                    MlbApiError::UnexpectedResponse(msg.clone())
                }
                MockError::NetworkUnavailable => MlbApiError::NetworkUnavailable,
            });
        }

        let json = self.responses.get(route).ok_or_else(|| {
            MlbApiError::UnexpectedResponse(format!(
                "MockMlbClient: no fixture loaded for route \"{route}\". \
                 Call .with_response(\"{route}\", json) before this method."
            ))
        })?;

        serde_json::from_str(json).map_err(MlbApiError::from)
    }
}

impl Default for MockMlbClient {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Endpoint methods — signatures must match MlbClient exactly.
//
// When adding a new endpoint to MlbClient, add a corresponding method here.
// The route key must match the table in the struct doc comment above.
// ---------------------------------------------------------------------------

impl MockMlbClient {
    pub async fn get_schedule_today(&self) -> Result<ScheduleResponse, MlbApiError> {
        self.resolve("schedule", vec![])
    }

    pub async fn get_schedule_for_date(
        &self,
        team_id: impl Into<TeamId>,
        date: &str,
    ) -> Result<ScheduleResponse, MlbApiError> {
        let tid = team_id.into();
        self.resolve("schedule", vec![format!("teamId={tid}"), format!("date={date}")])
    }

    pub async fn get_schedule(
        &self,
        params: &crate::client::ScheduleParams,
    ) -> Result<ScheduleResponse, MlbApiError> {
        self.resolve("schedule", vec![format!("params={params:?}")])
    }

    pub async fn get_live_game(&self, game_pk: u64) -> Result<LiveGameFeed, MlbApiError> {
        self.resolve("live_game", vec![format!("gamePk={game_pk}")])
    }

    pub async fn get_live_game_fields(
        &self,
        game_pk: u64,
        fields: &str,
    ) -> Result<LiveGameFeed, MlbApiError> {
        self.resolve(
            "live_game",
            vec![format!("gamePk={game_pk}"), format!("fields={fields}")],
        )
    }

    pub async fn get_live_game_diff_patch(
        &self,
        game_pk: u64,
        start_time_code: &str,
    ) -> Result<DiffPatchResponse, MlbApiError> {
        self.resolve(
            "live_game_diff_patch",
            vec![
                format!("gamePk={game_pk}"),
                format!("startTimeCode={start_time_code}"),
            ],
        )
    }

    pub async fn get_live_game_timestamps(
        &self,
        game_pk: u64,
    ) -> Result<TimestampsResponse, MlbApiError> {
        self.resolve("live_game_timestamps", vec![format!("gamePk={game_pk}")])
    }

    pub async fn get_standings(
        &self,
        league_id: impl Into<LeagueId>,
        season: u32,
    ) -> Result<StandingsResponse, MlbApiError> {
        let lid = league_id.into();
        self.resolve("standings", vec![format!("leagueId={lid}"), format!("season={season}")])
    }

    pub async fn get_teams(
        &self,
        season: u32,
    ) -> Result<TeamsResponse, MlbApiError> {
        self.resolve("teams", vec![format!("season={season}")])
    }

    pub async fn get_roster(
        &self,
        team_id: impl Into<TeamId>,
        roster_type: &str,
        season: u32,
    ) -> Result<RosterResponse, MlbApiError> {
        let tid = team_id.into();
        self.resolve(
            "roster",
            vec![
                format!("teamId={tid}"),
                format!("rosterType={roster_type}"),
                format!("season={season}"),
            ],
        )
    }

    pub async fn get_person(&self, player_id: u32) -> Result<PeopleResponse, MlbApiError> {
        self.resolve("person", vec![format!("playerId={player_id}")])
    }

    pub async fn get_linescore(&self, game_pk: u64) -> Result<LinescoreResponse, MlbApiError> {
        self.resolve("linescore", vec![format!("gamePk={game_pk}")])
    }

    pub async fn get_boxscore(&self, game_pk: u64) -> Result<BoxscoreResponse, MlbApiError> {
        self.resolve("boxscore", vec![format!("gamePk={game_pk}")])
    }

    pub async fn get_play_by_play(
        &self,
        game_pk: u64,
    ) -> Result<PlayByPlayResponse, MlbApiError> {
        self.resolve("play_by_play", vec![format!("gamePk={game_pk}")])
    }

    pub async fn get_win_probability(
        &self,
        game_pk: u64,
    ) -> Result<Vec<WinProbabilityEntry>, MlbApiError> {
        self.resolve("win_probability", vec![format!("gamePk={game_pk}")])
    }

    pub async fn get_game_content(&self, game_pk: u64) -> Result<GameContent, MlbApiError> {
        self.resolve("game_content", vec![format!("gamePk={game_pk}")])
    }

    pub async fn get_context_metrics(
        &self,
        game_pk: u64,
    ) -> Result<ContextMetrics, MlbApiError> {
        self.resolve("context_metrics", vec![format!("gamePk={game_pk}")])
    }

    // swagger: GET /v1/game/changes — bulk feed, no gamePk param
    pub async fn get_game_changes(
        &self,
        updated_since: &str,
    ) -> Result<GameChangesResponse, MlbApiError> {
        self.resolve("game_changes", vec![format!("updatedSince={updated_since}")])
    }

    pub async fn get_game_pace(
        &self,
        season: u32,
    ) -> Result<GamePaceResponse, MlbApiError> {
        self.resolve("game_pace", vec![format!("season={season}")])
    }

    pub async fn get_high_low(
        &self,
        sort_stat: &str,
        season: u32,
    ) -> Result<HighLowResponse, MlbApiError> {
        self.resolve(
            "high_low",
            vec![format!("sortStat={sort_stat}"), format!("season={season}")],
        )
    }

    pub async fn get_venues(&self, venue_id: u32) -> Result<VenuesResponse, MlbApiError> {
        self.resolve("venues", vec![format!("venueId={venue_id}")])
    }

    pub async fn get_leagues(&self) -> Result<LeagueResponse, MlbApiError> {
        self.resolve("leagues", vec![])
    }

    pub async fn get_divisions(&self) -> Result<DivisionsResponse, MlbApiError> {
        self.resolve("divisions", vec![])
    }

    pub async fn get_conferences(&self) -> Result<ConferencesResponse, MlbApiError> {
        self.resolve("conferences", vec![])
    }

    pub async fn get_sports(&self) -> Result<SportsResponse, MlbApiError> {
        self.resolve("sports", vec![])
    }

    pub async fn get_season(
        &self,
        season: u32,
        sport_id: u32,
    ) -> Result<SeasonsResponse, MlbApiError> {
        self.resolve("season", vec![format!("season={season}"), format!("sportId={sport_id}")])
    }

    pub async fn get_all_seasons(&self, sport_id: u32) -> Result<SeasonsResponse, MlbApiError> {
        self.resolve("seasons", vec![format!("sportId={sport_id}")])
    }

    pub async fn get_stats(
        &self,
        params: &crate::client::StatsParams,
    ) -> Result<StatsResponse, MlbApiError> {
        self.resolve("stats", vec![format!("params={params:?}")])
    }

    pub async fn get_team_stats(
        &self,
        team_id: impl Into<TeamId>,
        params: &crate::client::StatsParams,
    ) -> Result<StatsResponse, MlbApiError> {
        let tid = team_id.into();
        self.resolve("team_stats", vec![format!("teamId={tid}"), format!("params={params:?}")])
    }

    pub async fn get_league_leaders(
        &self,
        params: &crate::client::LeaderParams,
    ) -> Result<LeagueLeadersResponse, MlbApiError> {
        self.resolve("league_leaders", vec![format!("params={params:?}")])
    }

    pub async fn get_team_leaders(
        &self,
        team_id: impl Into<TeamId>,
        params: &crate::client::LeaderParams,
    ) -> Result<TeamLeadersResponse, MlbApiError> {
        let tid = team_id.into();
        self.resolve("team_leaders", vec![format!("teamId={tid}"), format!("params={params:?}")])
    }

    pub async fn get_stat_streaks(
        &self,
        params: &crate::client::StatStreakParams,
    ) -> Result<serde_json::Value, MlbApiError> {
        self.resolve("stat_streaks", vec![format!("params={params:?}")])
    }

    pub async fn get_stats_metrics(
        &self,
        season: u32,
    ) -> Result<serde_json::Value, MlbApiError> {
        self.resolve("stats_metrics", vec![format!("season={season}")])
    }

    pub async fn get_attendance(
        &self,
        team_id: impl Into<TeamId>,
        season: u32,
    ) -> Result<AttendanceResponse, MlbApiError> {
        let tid = team_id.into();
        self.resolve("attendance", vec![format!("teamId={tid}"), format!("season={season}")])
    }

    pub async fn get_coaches(&self, team_id: impl Into<TeamId>, season: u32) -> Result<CoachesResponse, MlbApiError> {
        let tid = team_id.into();
        self.resolve("coaches", vec![format!("teamId={tid}"), format!("season={season}")])
    }

    pub async fn get_umpires(&self) -> Result<UmpiresResponse, MlbApiError> {
        self.resolve("umpires", vec![])
    }

    // swagger: GET /v1/jobs/umpires/games/{umpireId} — umpireId is a path param
    pub async fn get_umpire_schedule(
        &self,
        umpire_id: u32,
        season: u32,
    ) -> Result<UmpireScheduleResponse, MlbApiError> {
        self.resolve(
            "umpire_schedule",
            vec![format!("umpireId={umpire_id}"), format!("season={season}")],
        )
    }

    pub async fn get_meta(&self, meta_type: MetaType) -> Result<Vec<MetaEntry>, MlbApiError> {
        self.resolve("meta", vec![format!("type={meta_type:?}")])
    }
}