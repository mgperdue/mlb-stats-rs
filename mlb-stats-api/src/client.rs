//! HTTP client for the MLB Stats API.
//!
//! The primary entry point is [`MlbClient`]. All endpoint methods are `async`
//! and return strongly-typed response structs from [`crate::models`].
//!
//! # Basic usage
//!
//! ```rust,no_run
//! use mlb_stats_api::{MlbClient, models::common::{LeagueId, TeamId}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = MlbClient::new();
//!
//!     let standings = client.get_standings(LeagueId(103), 2026).await?;
//!     let schedule  = client.get_schedule_for_date(TeamId(140), "2026-04-14").await?;
//!     Ok(())
//! }
//! ```
//!
//! When using the `ballpark` crate, named enums convert automatically via `Into`:
//!
//! ```rust,ignore
//! use ballpark::{League, Team};
//!
//! let standings = client.get_standings(League::AmericanLeague, 2026).await?;
//! let schedule  = client.get_schedule_for_date(Team::Rangers, "2026-04-14").await?;
//! ```

use serde::de::DeserializeOwned;
use tracing::instrument;

use crate::{
    error::MlbApiError,
    models::{
        attendance::AttendanceResponse,
        common::{LeagueId, TeamId},
        game::{
            BoxscoreResponse, ContextMetrics, GameChangesResponse, GameContent,
            GamePaceResponse, HighLowResponse, LinescoreResponse, WinProbabilityEntry,
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
    },
};

const DEFAULT_BASE_URL: &str = "https://statsapi.mlb.com/api/v1";
const DEFAULT_LIVE_BASE_URL: &str = "https://statsapi.mlb.com/api/v1.1";

// ---------------------------------------------------------------------------
// Parameter builder types
// ---------------------------------------------------------------------------

/// Parameters for [`MlbClient::get_schedule`].
///
/// Build with [`ScheduleParams::new`] and chain optional methods.
///
/// ```rust
/// use mlb_stats_api::client::ScheduleParams;
///
/// let params = ScheduleParams::new()
///     .date("2026-04-15")
///     .team_id(140u32)
///     .hydrate("linescore,decisions");
/// ```
#[derive(Debug, Default, Clone)]
pub struct ScheduleParams {
    pub date: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub team_id: Option<u32>,
    pub opponent_id: Option<u32>,
    pub game_types: Option<String>,
    pub hydrate: Option<String>,
    pub fields: Option<String>,
}

impl ScheduleParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Single date in `YYYY-MM-DD` format. Converted to `MM/DD/YYYY` for the API.
    pub fn date(mut self, d: &str) -> Self {
        self.date = Some(d.to_string());
        self
    }

    /// Start of a date range in `YYYY-MM-DD` format.
    pub fn start_date(mut self, d: &str) -> Self {
        self.start_date = Some(d.to_string());
        self
    }

    /// End of a date range in `YYYY-MM-DD` format.
    pub fn end_date(mut self, d: &str) -> Self {
        self.end_date = Some(d.to_string());
        self
    }

    pub fn team_id(mut self, id: impl Into<TeamId>) -> Self {
        self.team_id = Some(id.into().0);
        self
    }

    pub fn opponent_id(mut self, id: impl Into<TeamId>) -> Self {
        self.opponent_id = Some(id.into().0);
        self
    }

    /// Comma-separated game type codes, e.g. `"R"`, `"P"`, `"W"`.
    pub fn game_types(mut self, gt: &str) -> Self {
        self.game_types = Some(gt.to_string());
        self
    }

    /// Comma-separated hydration keys, e.g. `"linescore,decisions,weather"`.
    pub fn hydrate(mut self, h: &str) -> Self {
        self.hydrate = Some(h.to_string());
        self
    }

    pub fn fields(mut self, f: &str) -> Self {
        self.fields = Some(f.to_string());
        self
    }

    /// Convert a `YYYY-MM-DD` date string to the `MM/DD/YYYY` format the
    /// schedule endpoint requires.
    fn fmt_date(d: &str) -> String {
        // Best-effort: if the string already contains '/' assume it's already
        // in the right format (caller is using MM/DD/YYYY directly).
        if d.contains('/') {
            return d.to_string();
        }
        let parts: Vec<&str> = d.splitn(3, '-').collect();
        if parts.len() == 3 {
            format!("{}/{}/{}", parts[1], parts[2], parts[0])
        } else {
            d.to_string()
        }
    }

    fn to_query_string(&self) -> String {
        let mut parts = vec!["sportId=1".to_string()];
        if let Some(ref d) = self.date {
            parts.push(format!("date={}", Self::fmt_date(d)));
        }
        if let Some(ref d) = self.start_date {
            parts.push(format!("startDate={}", Self::fmt_date(d)));
        }
        if let Some(ref d) = self.end_date {
            parts.push(format!("endDate={}", Self::fmt_date(d)));
        }
        if let Some(id) = self.team_id {
            parts.push(format!("teamId={id}"));
        }
        if let Some(id) = self.opponent_id {
            parts.push(format!("opponentId={id}"));
        }
        if let Some(ref gt) = self.game_types {
            parts.push(format!("gameTypes={gt}"));
        }
        if let Some(ref h) = self.hydrate {
            parts.push(format!("hydrate={h}"));
        }
        if let Some(ref f) = self.fields {
            parts.push(format!("fields={f}"));
        }
        parts.join("&")
    }
}

/// Parameters for [`MlbClient::get_stats`] and [`MlbClient::get_team_stats`].
///
/// ```rust
/// use mlb_stats_api::client::StatsParams;
///
/// let params = StatsParams::new("season", "hitting")
///     .season(2026)
///     .player_pool("All");
/// ```
#[derive(Debug, Clone)]
pub struct StatsParams {
    pub stat_type: String,
    pub group: String,
    pub season: Option<u32>,
    pub player_pool: Option<String>,
    pub hydrate: Option<String>,
    pub fields: Option<String>,
}

impl StatsParams {
    /// `stat_type`: e.g. `"season"`, `"career"`, `"yearByYear"`.
    /// `group`: e.g. `"hitting"`, `"pitching"`, `"fielding"`.
    pub fn new(stat_type: &str, group: &str) -> Self {
        Self {
            stat_type: stat_type.to_string(),
            group: group.to_string(),
            season: None,
            player_pool: None,
            hydrate: None,
            fields: None,
        }
    }

    pub fn season(mut self, s: u32) -> Self {
        self.season = Some(s);
        self
    }

    /// `"All"`, `"Qualified"`, `"Rookies"`, etc.
    pub fn player_pool(mut self, p: &str) -> Self {
        self.player_pool = Some(p.to_string());
        self
    }

    pub fn hydrate(mut self, h: &str) -> Self {
        self.hydrate = Some(h.to_string());
        self
    }

    pub fn fields(mut self, f: &str) -> Self {
        self.fields = Some(f.to_string());
        self
    }

    fn to_query_string(&self) -> String {
        let mut parts = vec![
            format!("stats={}", self.stat_type),
            format!("group={}", self.group),
        ];
        if let Some(s) = self.season {
            parts.push(format!("season={s}"));
        }
        if let Some(ref pp) = self.player_pool {
            parts.push(format!("playerPool={pp}"));
        }
        if let Some(ref h) = self.hydrate {
            parts.push(format!("hydrate={h}"));
        }
        if let Some(ref f) = self.fields {
            parts.push(format!("fields={f}"));
        }
        parts.join("&")
    }
}

/// Parameters for [`MlbClient::get_league_leaders`] and
/// [`MlbClient::get_team_leaders`].
///
/// ```rust
/// use mlb_stats_api::client::LeaderParams;
///
/// let params = LeaderParams::new("homeRuns").season(2026);
/// ```
#[derive(Debug, Clone)]
pub struct LeaderParams {
    pub leader_categories: String,
    pub season: Option<u32>,
    pub player_pool: Option<String>,
    pub stat_group: Option<String>,
    pub sport_id: Option<u32>,
    pub hydrate: Option<String>,
    pub fields: Option<String>,
}

impl LeaderParams {
    /// `leader_categories`: comma-separated stat names, e.g.
    /// `"homeRuns"`, `"battingAverage,onBasePlusSlugging"`.
    pub fn new(leader_categories: &str) -> Self {
        Self {
            leader_categories: leader_categories.to_string(),
            season: None,
            player_pool: None,
            stat_group: None,
            sport_id: None,
            hydrate: None,
            fields: None,
        }
    }

    pub fn season(mut self, s: u32) -> Self {
        self.season = Some(s);
        self
    }

    pub fn player_pool(mut self, p: &str) -> Self {
        self.player_pool = Some(p.to_string());
        self
    }

    pub fn stat_group(mut self, g: &str) -> Self {
        self.stat_group = Some(g.to_string());
        self
    }

    pub fn sport_id(mut self, id: u32) -> Self {
        self.sport_id = Some(id);
        self
    }

    fn to_query_string(&self) -> String {
        let mut parts = vec![format!("leaderCategories={}", self.leader_categories)];
        if let Some(s) = self.season {
            parts.push(format!("season={s}"));
        }
        if let Some(ref pp) = self.player_pool {
            parts.push(format!("playerPool={pp}"));
        }
        if let Some(ref sg) = self.stat_group {
            parts.push(format!("statGroup={sg}"));
        }
        if let Some(id) = self.sport_id {
            parts.push(format!("sportId={id}"));
        }
        if let Some(ref h) = self.hydrate {
            parts.push(format!("hydrate={h}"));
        }
        if let Some(ref f) = self.fields {
            parts.push(format!("fields={f}"));
        }
        parts.join("&")
    }
}

/// Parameters for [`MlbClient::get_stat_streaks`].
///
/// ```rust
/// use mlb_stats_api::client::StatStreakParams;
///
/// let params = StatStreakParams::new("hittingStreakOverall", 2026);
/// ```
#[derive(Debug, Clone)]
pub struct StatStreakParams {
    pub streak_type: String,
    pub season: u32,
    pub sport_id: Option<u32>,
    pub game_type: Option<String>,
    pub limit: Option<u32>,
    pub fields: Option<String>,
}

impl StatStreakParams {
    pub fn new(streak_type: &str, season: u32) -> Self {
        Self {
            streak_type: streak_type.to_string(),
            season,
            sport_id: None,
            game_type: None,
            limit: None,
            fields: None,
        }
    }

    pub fn sport_id(mut self, id: u32) -> Self {
        self.sport_id = Some(id);
        self
    }

    pub fn game_type(mut self, gt: &str) -> Self {
        self.game_type = Some(gt.to_string());
        self
    }

    pub fn limit(mut self, n: u32) -> Self {
        self.limit = Some(n);
        self
    }

    fn to_query_string(&self) -> String {
        let mut parts = vec![
            format!("streakType={}", self.streak_type),
            format!("season={}", self.season),
        ];
        if let Some(id) = self.sport_id {
            parts.push(format!("sportId={id}"));
        }
        if let Some(ref gt) = self.game_type {
            parts.push(format!("gameType={gt}"));
        }
        if let Some(n) = self.limit {
            parts.push(format!("limit={n}"));
        }
        if let Some(ref f) = self.fields {
            parts.push(format!("fields={f}"));
        }
        parts.join("&")
    }
}

// ---------------------------------------------------------------------------
// MlbClient
// ---------------------------------------------------------------------------

/// Async HTTP client for the MLB Stats API (`statsapi.mlb.com`).
///
/// All methods are `async` and return `Result<_, MlbApiError>`. Construct
/// with [`MlbClient::new`]; override base URLs with
/// [`with_base_url`][Self::with_base_url] and
/// [`with_live_base_url`][Self::with_live_base_url] for testing.
///
/// # Example
///
/// ```rust,no_run
/// use mlb_stats_api::MlbClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = MlbClient::new();
///     let schedule = client.get_schedule_today().await?;
///     println!("{} games today", schedule.total_games.unwrap_or(0));
///     Ok(())
/// }
/// ```
pub struct MlbClient {
    http: reqwest::Client,
    /// Base URL for v1 endpoints. Override with [`with_base_url`][Self::with_base_url].
    base_url: String,
    /// Base URL for v1.1 live feed endpoints. Override with
    /// [`with_live_base_url`][Self::with_live_base_url].
    live_base_url: String,
}

impl MlbClient {
    /// Creates a new client pointing at the live MLB Stats API.
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            live_base_url: DEFAULT_LIVE_BASE_URL.to_string(),
        }
    }

    /// Override the v1 base URL — used in tests to point at a wiremock server.
    ///
    /// ```rust
    /// use mlb_stats_api::MlbClient;
    ///
    /// let client = MlbClient::new().with_base_url("http://localhost:8080/api/v1");
    /// ```
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    /// Override the v1.1 live feed base URL — used in tests.
    pub fn with_live_base_url(mut self, url: &str) -> Self {
        self.live_base_url = url.to_string();
        self
    }

    // -----------------------------------------------------------------------
    // Schedule
    // -----------------------------------------------------------------------

    /// Returns all MLB games scheduled today.
    ///
    /// Equivalent to `GET /api/v1/schedule?sportId=1&date={today}`.
    #[instrument(skip(self))]
    pub async fn get_schedule_today(&self) -> Result<ScheduleResponse, MlbApiError> {
        let today = chrono::Local::now().format("%m/%d/%Y").to_string();
        let url = format!("{}/schedule?sportId=1&date={today}", self.base_url);
        self.get_json(&url).await
    }

    /// Returns games for a specific team on a given date.
    ///
    /// `date` is `YYYY-MM-DD`. Converted to `MM/DD/YYYY` for the API.
    #[instrument(skip(self))]
    pub async fn get_schedule_for_date(
        &self,
        team_id: impl Into<TeamId>,
        date: &str,
    ) -> Result<ScheduleResponse, MlbApiError> {
        let team_id = team_id.into();
        let api_date = ScheduleParams::fmt_date(date);
        let url = format!(
            "{}/schedule?sportId=1&teamId={team_id}&date={api_date}",
            self.base_url
        );
        self.get_json(&url).await
    }

    /// Full-featured schedule query with optional parameters.
    ///
    /// ```rust,no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use mlb_stats_api::{MlbClient, client::ScheduleParams};
    ///
    /// let client = MlbClient::new();
    /// let params = ScheduleParams::new()
    ///     .start_date("2026-04-01")
    ///     .end_date("2026-04-30")
    ///     .team_id(140u32)
    ///     .hydrate("linescore,decisions");
    /// let schedule = client.get_schedule(&params).await?;
    /// # Ok(()) }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_schedule(
        &self,
        params: &ScheduleParams,
    ) -> Result<ScheduleResponse, MlbApiError> {
        let url = format!("{}/schedule?{}", self.base_url, params.to_query_string());
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Live game feed
    // -----------------------------------------------------------------------

    /// Full live game feed. Uses the v1.1 endpoint required for live data.
    ///
    /// `GET /api/v1.1/game/{gamePk}/feed/live`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_live_game(&self, game_pk: u64) -> Result<LiveGameFeed, MlbApiError> {
        let url = format!("{}/game/{game_pk}/feed/live", self.live_base_url);
        self.get_json(&url).await
    }

    /// Live game feed filtered to specific fields. Useful for polling — reduces
    /// response size when only a subset of data is needed.
    ///
    /// `fields` is a comma-separated list of JSON field names,
    /// e.g. `"gamePk,gameData,status,abstractGameState"`.
    ///
    /// `GET /api/v1.1/game/{gamePk}/feed/live?fields=...`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_live_game_fields(
        &self,
        game_pk: u64,
        fields: &str,
    ) -> Result<LiveGameFeed, MlbApiError> {
        let url = format!(
            "{}/game/{game_pk}/feed/live?fields={fields}",
            self.live_base_url
        );
        self.get_json(&url).await
    }

    /// Returns only the changes to the live feed since a given timestamp.
    /// Designed for efficient polling — returns a minimal diff rather than
    /// the full feed.
    ///
    /// `start_time_code` and `end_time_code` are in `YYYYMMDD_HHMMSS` format.
    ///
    /// `GET /api/v1.1/game/{gamePk}/feed/live/diffPatch`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_live_game_diff_patch(
        &self,
        game_pk: u64,
        start_time_code: &str,
        end_time_code: Option<&str>,
    ) -> Result<DiffPatchResponse, MlbApiError> {
        let mut url = format!(
            "{}/game/{game_pk}/feed/live/diffPatch?startTimecode={start_time_code}",
            self.live_base_url
        );
        if let Some(end) = end_time_code {
            url.push_str(&format!("&endTimecode={end}"));
        }
        self.get_json(&url).await
    }

    /// Returns all available timestamps for a game's live feed.
    /// Use these timestamps with [`get_live_game_diff_patch`][Self::get_live_game_diff_patch].
    ///
    /// `GET /api/v1.1/game/{gamePk}/feed/live/timestamps`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_live_game_timestamps(
        &self,
        game_pk: u64,
    ) -> Result<TimestampsResponse, MlbApiError> {
        let url = format!(
            "{}/game/{game_pk}/feed/live/timestamps",
            self.live_base_url
        );
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Standings
    // -----------------------------------------------------------------------

    /// Returns standings for a league and season.
    ///
    /// `GET /api/v1/standings?leagueId={leagueId}&season={season}`
    ///
    /// For both leagues in one call, use the raw [`get`][Self::get] escape hatch
    /// with `leagueId=103,104`, or call this method twice.
    #[instrument(skip(self))]
    pub async fn get_standings(
        &self,
        league_id: impl Into<LeagueId>,
        season: u32,
    ) -> Result<StandingsResponse, MlbApiError> {
        let league_id = league_id.into();
        let url = format!(
            "{}/standings?leagueId={league_id}&season={season}",
            self.base_url
        );
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Teams & Roster
    // -----------------------------------------------------------------------

    /// Returns all MLB teams for a given season.
    ///
    /// `GET /api/v1/teams?sportIds=1&season={season}`
    #[instrument(skip(self))]
    pub async fn get_teams(&self, season: u32) -> Result<TeamsResponse, MlbApiError> {
        let url = format!("{}/teams?sportIds=1&season={season}", self.base_url);
        self.get_json(&url).await
    }

    /// Returns the roster for a team.
    ///
    /// `roster_type`: `"active"`, `"40Man"`, `"fullRoster"`, `"depthChart"`, etc.
    /// Use [`get_meta(MetaType::RosterTypes)`][Self::get_meta] for the full list.
    ///
    /// `GET /api/v1/teams/{teamId}/roster?rosterType={type}&season={season}`
    #[instrument(skip(self))]
    pub async fn get_roster(
        &self,
        team_id: impl Into<TeamId>,
        roster_type: &str,
        season: u32,
    ) -> Result<RosterResponse, MlbApiError> {
        let team_id = team_id.into();
        let url = format!(
            "{}/teams/{team_id}/roster?rosterType={roster_type}&season={season}",
            self.base_url
        );
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // People
    // -----------------------------------------------------------------------

    /// Returns player bio and, if hydrated, stats.
    ///
    /// To include stats, pass a `hydrate` string:
    /// `"stats(group=hitting,type=season)"`.
    ///
    /// `GET /api/v1/people/{personId}[?hydrate=...]`
    #[instrument(skip(self))]
    pub async fn get_person(
        &self,
        player_id: u32,
        hydrate: Option<&str>,
    ) -> Result<PeopleResponse, MlbApiError> {
        let mut url = format!("{}/people/{player_id}", self.base_url);
        if let Some(h) = hydrate {
            url.push_str(&format!("?hydrate={h}"));
        }
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Standalone game endpoints
    // -----------------------------------------------------------------------

    /// Inning-by-inning linescore for a game.
    ///
    /// `GET /api/v1/game/{gamePk}/linescore`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_linescore(&self, game_pk: u64) -> Result<LinescoreResponse, MlbApiError> {
        let url = format!("{}/game/{game_pk}/linescore", self.base_url);
        self.get_json(&url).await
    }

    /// Detailed boxscore with player stats for a game.
    ///
    /// `GET /api/v1/game/{gamePk}/boxscore`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_boxscore(&self, game_pk: u64) -> Result<BoxscoreResponse, MlbApiError> {
        let url = format!("{}/game/{game_pk}/boxscore", self.base_url);
        self.get_json(&url).await
    }

    /// Play-by-play for a game.
    ///
    /// `GET /api/v1/game/{gamePk}/playByPlay`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_play_by_play(
        &self,
        game_pk: u64,
    ) -> Result<PlayByPlayResponse, MlbApiError> {
        let url = format!("{}/game/{game_pk}/playByPlay", self.base_url);
        self.get_json(&url).await
    }

    /// Win probability data for each play of a game.
    ///
    /// `GET /api/v1/game/{gamePk}/winProbability`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_win_probability(
        &self,
        game_pk: u64,
    ) -> Result<Vec<WinProbabilityEntry>, MlbApiError> {
        let url = format!("{}/game/{game_pk}/winProbability", self.base_url);
        self.get_json(&url).await
    }

    /// Editorial content (highlights, articles) for a game.
    ///
    /// `GET /api/v1/game/{gamePk}/content`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_game_content(&self, game_pk: u64) -> Result<GameContent, MlbApiError> {
        let url = format!("{}/game/{game_pk}/content", self.base_url);
        self.get_json(&url).await
    }

    /// Context metrics (leverage index, base-out state, etc.) for a game.
    ///
    /// `GET /api/v1/game/{gamePk}/contextMetrics`
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_context_metrics(
        &self,
        game_pk: u64,
    ) -> Result<ContextMetrics, MlbApiError> {
        let url = format!("{}/game/{game_pk}/contextMetrics", self.base_url);
        self.get_json(&url).await
    }

    /// Umpire crew for a game.
    ///
    /// `GET /api/v1/game/{gamePk}/officials`
    ///
    /// Note: this endpoint is not in the public swagger but is confirmed
    /// to exist and return umpire data in practice.
    #[instrument(skip(self), fields(game_pk))]
    pub async fn get_game_officials(
        &self,
        game_pk: u64,
    ) -> Result<UmpiresResponse, MlbApiError> {
        let url = format!("{}/game/{game_pk}/officials", self.base_url);
        self.get_json(&url).await
    }

    /// Changes to game data across all games since a given timestamp.
    ///
    /// This is a **bulk** feed — it returns changes for all games, not
    /// a specific game. Use the returned data to decide which games need
    /// a full live feed refresh.
    ///
    /// `updated_since` format: ISO 8601, e.g. `"2026-04-15T19:00:00Z"`.
    ///
    /// `GET /api/v1/game/changes?updatedSince={timestamp}`
    #[instrument(skip(self))]
    pub async fn get_game_changes(
        &self,
        updated_since: &str,
    ) -> Result<GameChangesResponse, MlbApiError> {
        let url = format!(
            "{}/game/changes?updatedSince={updated_since}",
            self.base_url
        );
        self.get_json(&url).await
    }

    /// Game pace statistics for a season.
    ///
    /// `GET /api/v1/gamePace?season={season}`
    #[instrument(skip(self))]
    pub async fn get_game_pace(&self, season: u32) -> Result<GamePaceResponse, MlbApiError> {
        let url = format!("{}/gamePace?season={season}", self.base_url);
        self.get_json(&url).await
    }

    /// High/low leaders for a stat within an org type.
    ///
    /// `org_type`: `"team"`, `"player"`, `"division"`, `"league"`, etc.
    /// `sort_stat`: stat name, e.g. `"homeRuns"`, `"battingAverage"`.
    ///
    /// `GET /api/v1/highLow/{orgType}?sortStat={stat}&season={season}`
    #[instrument(skip(self))]
    pub async fn get_high_low(
        &self,
        org_type: &str,
        sort_stat: &str,
        season: u32,
    ) -> Result<HighLowResponse, MlbApiError> {
        let url = format!(
            "{}/highLow/{org_type}?sortStat={sort_stat}&season={season}",
            self.base_url
        );
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Venue
    // -----------------------------------------------------------------------

    /// Returns venue information, optionally with field dimensions.
    ///
    /// Pass `hydrate: Some("fieldInfo")` to include field dimensions.
    ///
    /// `GET /api/v1/venues?venueIds={id}&sportIds=1[&hydrate=fieldInfo]`
    #[instrument(skip(self))]
    pub async fn get_venues(
        &self,
        venue_id: u32,
        hydrate: Option<&str>,
    ) -> Result<VenuesResponse, MlbApiError> {
        let mut url = format!(
            "{}/venues?venueIds={venue_id}&sportIds=1",
            self.base_url
        );
        if let Some(h) = hydrate {
            url.push_str(&format!("&hydrate={h}"));
        }
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // League / Division / Sport / Conference
    // -----------------------------------------------------------------------

    /// Returns AL and NL league information.
    ///
    /// `GET /api/v1/league?sportId=1&leagueIds=103,104`
    ///
    /// For a different sport, use the raw [`get`][Self::get] escape hatch.
    #[instrument(skip(self))]
    pub async fn get_leagues(&self) -> Result<LeagueResponse, MlbApiError> {
        let url = format!("{}/league?sportId=1&leagueIds=103,104", self.base_url);
        self.get_json(&url).await
    }

    /// Returns all MLB divisions.
    ///
    /// `GET /api/v1/divisions?sportId=1`
    #[instrument(skip(self))]
    pub async fn get_divisions(&self) -> Result<DivisionsResponse, MlbApiError> {
        let url = format!("{}/divisions?sportId=1", self.base_url);
        self.get_json(&url).await
    }

    /// Returns conference data (used for minor leagues; less relevant for MLB).
    ///
    /// `GET /api/v1/conferences`
    #[instrument(skip(self))]
    pub async fn get_conferences(&self) -> Result<ConferencesResponse, MlbApiError> {
        let url = format!("{}/conferences", self.base_url);
        self.get_json(&url).await
    }

    /// Returns all sports tracked by the Stats API.
    ///
    /// `GET /api/v1/sports`
    #[instrument(skip(self))]
    pub async fn get_sports(&self) -> Result<SportsResponse, MlbApiError> {
        let url = format!("{}/sports", self.base_url);
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Season
    // -----------------------------------------------------------------------

    /// Returns date information for a specific season.
    ///
    /// `GET /api/v1/seasons/{season}?sportId={sportId}`
    #[instrument(skip(self))]
    pub async fn get_season(
        &self,
        season: u32,
        sport_id: u32,
    ) -> Result<SeasonsResponse, MlbApiError> {
        let url = format!(
            "{}/seasons/{season}?sportId={sport_id}",
            self.base_url
        );
        self.get_json(&url).await
    }

    /// Returns date information for all seasons of a sport.
    ///
    /// `GET /api/v1/seasons?sportId={sportId}&all=true`
    #[instrument(skip(self))]
    pub async fn get_all_seasons(&self, sport_id: u32) -> Result<SeasonsResponse, MlbApiError> {
        let url = format!(
            "{}/seasons?sportId={sport_id}&all=true",
            self.base_url
        );
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Stats
    // -----------------------------------------------------------------------

    /// Returns player statistics.
    ///
    /// ```rust,no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use mlb_stats_api::{MlbClient, client::StatsParams};
    ///
    /// let client = MlbClient::new();
    /// let params = StatsParams::new("season", "hitting")
    ///     .season(2026)
    ///     .player_pool("All");
    /// let stats = client.get_stats(&params).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// `GET /api/v1/stats?stats={type}&group={group}[&season=...&playerPool=...]`
    #[instrument(skip(self))]
    pub async fn get_stats(&self, params: &StatsParams) -> Result<StatsResponse, MlbApiError> {
        let url = format!("{}/stats?{}", self.base_url, params.to_query_string());
        self.get_json(&url).await
    }

    /// Returns team statistics.
    ///
    /// `GET /api/v1/teams/{teamId}/stats?season={season}&statGroup={group}`
    #[instrument(skip(self))]
    pub async fn get_team_stats(
        &self,
        team_id: impl Into<TeamId>,
        params: &StatsParams,
    ) -> Result<StatsResponse, MlbApiError> {
        let team_id = team_id.into();
        let url = format!(
            "{}/teams/{team_id}/stats?{}",
            self.base_url,
            params.to_query_string()
        );
        self.get_json(&url).await
    }

    /// Returns league-wide statistical leaders.
    ///
    /// ```rust,no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use mlb_stats_api::{MlbClient, client::LeaderParams};
    ///
    /// let client = MlbClient::new();
    /// let params = LeaderParams::new("homeRuns").season(2026);
    /// let leaders = client.get_league_leaders(&params).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// `GET /api/v1/stats/leaders?leaderCategories={cats}&season={season}`
    #[instrument(skip(self))]
    pub async fn get_league_leaders(
        &self,
        params: &LeaderParams,
    ) -> Result<LeagueLeadersResponse, MlbApiError> {
        let url = format!(
            "{}/stats/leaders?{}",
            self.base_url,
            params.to_query_string()
        );
        self.get_json(&url).await
    }

    /// Returns statistical leaders for a specific team.
    ///
    /// `GET /api/v1/teams/{teamId}/leaders?leaderCategories={cats}&season={season}`
    #[instrument(skip(self))]
    pub async fn get_team_leaders(
        &self,
        team_id: impl Into<TeamId>,
        params: &LeaderParams,
    ) -> Result<TeamLeadersResponse, MlbApiError> {
        let team_id = team_id.into();
        let url = format!(
            "{}/teams/{team_id}/leaders?{}",
            self.base_url,
            params.to_query_string()
        );
        self.get_json(&url).await
    }

    /// Returns current hitting/on-base streak leaders.
    ///
    /// `GET /api/v1/stats/streaks?streakType={type}&season={season}`
    ///
    /// Note: the response schema is undocumented in the swagger; the field
    /// `streaks` is typed as `serde_json::Value` until a fixture confirms
    /// the shape.
    #[instrument(skip(self))]
    pub async fn get_stat_streaks(
        &self,
        params: &StatStreakParams,
    ) -> Result<serde_json::Value, MlbApiError> {
        let url = format!(
            "{}/stats/streaks?{}",
            self.base_url,
            params.to_query_string()
        );
        self.get_json(&url).await
    }

    /// Returns stats metrics definitions for a season.
    ///
    /// Note: the response schema is undocumented in the swagger; typed as
    /// `serde_json::Value` until a fixture confirms the shape.
    ///
    /// `GET /api/v1/stats/metrics?seasons={season}`
    #[instrument(skip(self))]
    pub async fn get_stats_metrics(
        &self,
        season: u32,
    ) -> Result<serde_json::Value, MlbApiError> {
        let url = format!("{}/stats/metrics?seasons={season}", self.base_url);
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Attendance
    // -----------------------------------------------------------------------

    /// Returns attendance data for a team and season.
    ///
    /// Note: the `records` field in the response is typed as
    /// `serde_json::Value` — the swagger schema for this type is empty.
    /// Revisit once fixture data confirms the shape.
    ///
    /// `GET /api/v1/attendance?teamId={teamId}&season={season}`
    #[instrument(skip(self))]
    pub async fn get_attendance(
        &self,
        team_id: impl Into<TeamId>,
        season: u32,
    ) -> Result<AttendanceResponse, MlbApiError> {
        let team_id = team_id.into();
        let url = format!(
            "{}/attendance?teamId={team_id}&season={season}",
            self.base_url
        );
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Coaches / Umpires
    // -----------------------------------------------------------------------

    /// Returns the coaching staff for a team and season.
    ///
    /// `GET /api/v1/teams/{teamId}/coaches?season={season}`
    #[instrument(skip(self))]
    pub async fn get_coaches(
        &self,
        team_id: impl Into<TeamId>,
        season: u32,
    ) -> Result<CoachesResponse, MlbApiError> {
        let team_id = team_id.into();
        let url = format!(
            "{}/teams/{team_id}/coaches?season={season}",
            self.base_url
        );
        self.get_json(&url).await
    }

    /// Returns the list of active MLB umpires.
    ///
    /// `GET /api/v1/jobs/umpires`
    #[instrument(skip(self))]
    pub async fn get_umpires(&self) -> Result<UmpiresResponse, MlbApiError> {
        let url = format!("{}/jobs/umpires", self.base_url);
        self.get_json(&url).await
    }

    /// Returns the game schedule for a specific umpire.
    ///
    /// `GET /api/v1/jobs/umpires/games/{umpireId}?season={season}`
    #[instrument(skip(self))]
    pub async fn get_umpire_schedule(
        &self,
        umpire_id: u32,
        season: u32,
    ) -> Result<UmpireScheduleResponse, MlbApiError> {
        let url = format!(
            "{}/jobs/umpires/games/{umpire_id}?season={season}",
            self.base_url
        );
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Meta
    // -----------------------------------------------------------------------

    /// Returns reference data for a given meta type.
    ///
    /// Useful for enumerating valid values for parameters like game types,
    /// pitch types, roster types, etc.
    ///
    /// ```rust,no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use mlb_stats_api::{MlbClient, MetaType};
    ///
    /// let client = MlbClient::new();
    /// let game_types = client.get_meta(MetaType::GameTypes).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// Note: this endpoint is not in the public swagger but is confirmed
    /// to exist. The response is typically `[{code, description}, ...]`
    /// but some meta types use different shapes; the three-shape fallback
    /// in this method handles the known variants.
    #[instrument(skip(self))]
    pub async fn get_meta(&self, meta_type: MetaType) -> Result<Vec<MetaEntry>, MlbApiError> {
        let type_str = meta_type.as_query_param();
        let url = format!("{}/meta?type={type_str}", self.base_url);
        self.get_json(&url).await
    }

    // -----------------------------------------------------------------------
    // Raw escape hatch
    // -----------------------------------------------------------------------

    /// Make a raw GET request to any URL and deserialize the response.
    ///
    /// Use this for endpoints not yet covered by typed methods, or for
    /// constructing custom queries (e.g. standings for both leagues at once).
    ///
    /// ```rust,no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// use mlb_stats_api::MlbClient;
    ///
    /// let client = MlbClient::new();
    ///
    /// // Both AL and NL standings in one call
    /// let standings: serde_json::Value = client
    ///     .get("https://statsapi.mlb.com/api/v1/standings?leagueId=103,104&season=2026")
    ///     .await?;
    /// # Ok(()) }
    /// ```
    #[instrument(skip(self))]
    pub async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, MlbApiError> {
        self.get_json(url).await
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    async fn get_json<T: DeserializeOwned>(&self, url: &str) -> Result<T, MlbApiError> {
        tracing::debug!(url, "GET");

        let response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(MlbApiError::Http)?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            return Err(MlbApiError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        if !response.status().is_success() {
            return Err(MlbApiError::UnexpectedResponse(format!(
                "HTTP {} for {url}",
                response.status(),
            )));
        }

        response
            .json::<T>()
            .await
            .map_err(|e| MlbApiError::Deserialize(e.into()))
    }
}

impl Default for MlbClient {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// team_id_from_abbr
// ---------------------------------------------------------------------------

/// Returns the [`TeamId`] for a given team abbreviation.
///
/// Matches case-insensitively. When using the `ballpark` crate, prefer
/// `"TEX".parse::<ballpark::Team>()` which also supports nicknames and
/// full team names.
///
/// # Errors
///
/// Returns [`MlbApiError::TeamNotFound`] for unrecognised abbreviations.
///
/// # Example
///
/// ```rust
/// use mlb_stats_api::{team_id_from_abbr, models::common::TeamId};
///
/// assert_eq!(team_id_from_abbr("TEX").unwrap(), TeamId(140));
/// assert_eq!(team_id_from_abbr("tex").unwrap(), TeamId(140));
/// assert!(team_id_from_abbr("XYZ").is_err());
/// ```
pub fn team_id_from_abbr(abbr: &str) -> Result<TeamId, MlbApiError> {
    let upper = abbr.trim().to_ascii_uppercase();
    let id = match upper.as_str() {
        // AL East
        "BAL" => 110,
        "BOS" => 111,
        "NYY" => 147,
        "TB"  => 139,
        "TOR" => 141,
        // AL Central
        "CWS" => 145,
        "CLE" => 114,
        "DET" => 116,
        "KC"  => 118,
        "MIN" => 142,
        // AL West
        "HOU" => 117,
        "LAA" => 108,
        "ATH" => 133,
        "SEA" => 136,
        "TEX" => 140,
        // NL East
        "ATL" => 144,
        "MIA" => 146,
        "NYM" => 121,
        "PHI" => 143,
        "WSH" => 120,
        // NL Central
        "CHC" => 112,
        "CIN" => 113,
        "MIL" => 158,
        "PIT" => 134,
        "STL" => 138,
        // NL West
        "ARI" => 109,
        "COL" => 115,
        "LAD" => 119,
        "SD"  => 135,
        "SF"  => 137,
        _ => return Err(MlbApiError::TeamNotFound(abbr.to_string())),
    };
    Ok(TeamId(id))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_params_date_conversion() {
        // YYYY-MM-DD → MM/DD/YYYY
        assert_eq!(ScheduleParams::fmt_date("2026-04-15"), "04/15/2026");
        assert_eq!(ScheduleParams::fmt_date("2026-11-01"), "11/01/2026");
    }

    #[test]
    fn schedule_params_passthrough_if_already_formatted() {
        // Already MM/DD/YYYY — should pass through unchanged
        assert_eq!(ScheduleParams::fmt_date("04/15/2026"), "04/15/2026");
    }

    #[test]
    fn schedule_params_query_string_includes_sport_id() {
        let params = ScheduleParams::new().date("2026-04-15");
        let qs = params.to_query_string();
        assert!(qs.contains("sportId=1"), "should always include sportId=1");
        assert!(qs.contains("date=04/15/2026"), "date should be formatted");
    }

    #[test]
    fn stats_params_query_string() {
        let params = StatsParams::new("season", "hitting").season(2026).player_pool("All");
        let qs = params.to_query_string();
        assert!(qs.contains("stats=season"));
        assert!(qs.contains("group=hitting"));
        assert!(qs.contains("season=2026"));
        assert!(qs.contains("playerPool=All"));
    }

    #[test]
    fn leader_params_query_string() {
        let params = LeaderParams::new("homeRuns").season(2026);
        let qs = params.to_query_string();
        assert!(qs.contains("leaderCategories=homeRuns"));
        assert!(qs.contains("season=2026"));
    }

    #[test]
    fn team_id_from_abbr_known() {
        assert_eq!(team_id_from_abbr("TEX").unwrap(), TeamId(140));
        assert_eq!(team_id_from_abbr("NYY").unwrap(), TeamId(147));
        assert_eq!(team_id_from_abbr("BOS").unwrap(), TeamId(111));
        assert_eq!(team_id_from_abbr("MIL").unwrap(), TeamId(158));
    }

    #[test]
    fn team_id_from_abbr_case_insensitive() {
        assert_eq!(team_id_from_abbr("tex").unwrap(), TeamId(140));
        assert_eq!(team_id_from_abbr("Tex").unwrap(), TeamId(140));
    }

    #[test]
    fn team_id_from_abbr_trims_whitespace() {
        assert_eq!(team_id_from_abbr("  TEX  ").unwrap(), TeamId(140));
    }

    #[test]
    fn team_id_from_abbr_unknown_returns_err() {
        assert!(matches!(
            team_id_from_abbr("XYZ"),
            Err(MlbApiError::TeamNotFound(_))
        ));
    }

    #[test]
    fn all_30_abbreviations_resolve() {
        let abbrs = [
            "BAL", "BOS", "NYY", "TB",  "TOR",
            "CWS", "CLE", "DET", "KC",  "MIN",
            "HOU", "LAA", "ATH", "SEA", "TEX",
            "ATL", "MIA", "NYM", "PHI", "WSH",
            "CHC", "CIN", "MIL", "PIT", "STL",
            "ARI", "COL", "LAD", "SD",  "SF",
        ];
        assert_eq!(abbrs.len(), 30);
        for abbr in abbrs {
            assert!(
                team_id_from_abbr(abbr).is_ok(),
                "{abbr} should resolve to a TeamId"
            );
        }
    }

    #[test]
    fn all_30_team_ids_are_unique() {
        let abbrs = [
            "BAL", "BOS", "NYY", "TB",  "TOR",
            "CWS", "CLE", "DET", "KC",  "MIN",
            "HOU", "LAA", "ATH", "SEA", "TEX",
            "ATL", "MIA", "NYM", "PHI", "WSH",
            "CHC", "CIN", "MIL", "PIT", "STL",
            "ARI", "COL", "LAD", "SD",  "SF",
        ];
        let mut ids: Vec<u32> = abbrs
            .iter()
            .map(|a| team_id_from_abbr(a).unwrap().0)
            .collect();
        let original_len = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(original_len, ids.len(), "duplicate team IDs");
    }
}