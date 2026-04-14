//! HTTP client for the MLB Stats API.
//!
//! # Usage
//!
//! ```rust
//! use mlb_stats_api::{MlbClient, models::{LeagueId, TeamId}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = MlbClient::new();
//!
//!     // Using raw typed IDs
//!     let standings = client.get_standings(LeagueId(103), 2026).await?;
//!     let schedule  = client.get_schedule_for_date(TeamId(140), "2026-04-14").await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! When using the `ballpark` crate, named enums convert automatically via `Into`:
//!
//! ```rust
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
        common::TeamId,
        live::LiveGameFeed,
        schedule::ScheduleResponse,
        standings::StandingsResponse,
        common::LeagueId,
    },
};

const DEFAULT_BASE_URL: &str = "https://statsapi.mlb.com/api/v1";
const LIVE_BASE_URL: &str = "https://statsapi.mlb.com/api/v1.1";

/// Async HTTP client for the MLB Stats API.
///
/// # Example
/// ```rust
/// use mlb_stats_api::MlbClient;
///
/// let client = MlbClient::new();
/// ```
pub struct MlbClient {
    http: reqwest::Client,
    /// Base URL for v1 endpoints. Override for testing.
    base_url: String,
    /// Base URL for v1.1 endpoints (live game feed). Override for testing.
    live_base_url: String,
}

impl MlbClient {
    /// Creates a new client pointing at the live MLB Stats API.
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: DEFAULT_BASE_URL.to_string(),
            live_base_url: LIVE_BASE_URL.to_string(),
        }
    }

    /// Overrides the base URL — used in tests to point at fixture servers
    /// or a mock.
    ///
    /// # Example
    /// ```rust
    /// use mlb_stats_api::MlbClient;
    ///
    /// let client = MlbClient::new()
    ///     .with_base_url("http://localhost:8080/api/v1");
    /// ```
    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    /// Overrides the live feed base URL — used in tests.
    pub fn with_live_base_url(mut self, url: &str) -> Self {
        self.live_base_url = url.to_string();
        self
    }

    // ----------------------------------------------------------------
    // Schedule
    // ----------------------------------------------------------------

    /// Returns all games scheduled for today across MLB.
    ///
    /// Equivalent to `GET /api/v1/schedule?sportId=1&date={today}`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use mlb_stats_api::MlbClient;
    /// let client = MlbClient::new();
    /// let schedule = client.get_schedule_today().await?;
    /// # Ok(()) }
    /// ```
    #[instrument(skip(self))]
    pub async fn get_schedule_today(
        &self,
    ) -> Result<ScheduleResponse, MlbApiError> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let url = format!(
            "{}/schedule?sportId=1&date={}",
            self.base_url, today
        );
        self.get_json(&url).await
    }

    /// Returns all games scheduled for a specific team on a given date.
    ///
    /// Equivalent to
    /// `GET /api/v1/schedule?sportId=1&teamId={teamId}&date={date}`.
    ///
    /// `team_id` accepts anything that converts to [`TeamId`] — including
    /// [`ballpark::Team`] variants when the `ballpark` crate is in use.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use mlb_stats_api::{MlbClient, models::TeamId};
    /// let client = MlbClient::new();
    ///
    /// // Using raw TeamId
    /// let schedule = client.get_schedule_for_date(TeamId(140), "2026-04-14").await?;
    ///
    /// // Using ballpark::Team (via Into<TeamId>)
    /// // let schedule = client.get_schedule_for_date(Team::Rangers, "2026-04-14").await?;
    /// # Ok(()) }
    /// ```
    #[instrument(skip(self), fields(team_id = %team_id.0))]
    pub async fn get_schedule_for_date(
        &self,
        team_id: impl Into<TeamId>,
        date: &str,
    ) -> Result<ScheduleResponse, MlbApiError> {
        let team_id = team_id.into();
        let url = format!(
            "{}/schedule?sportId=1&teamId={}&date={}",
            self.base_url, team_id, date
        );
        self.get_json(&url).await
    }

    // ----------------------------------------------------------------
    // Live game feed
    // ----------------------------------------------------------------

    /// Returns the full live game feed for a given game.
    ///
    /// Uses the v1.1 endpoint which is required for live feed data:
    /// `GET /api/v1.1/game/{gamePk}/feed/live`.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use mlb_stats_api::MlbClient;
    /// let client = MlbClient::new();
    /// let feed = client.get_live_game(747175).await?;
    /// # Ok(()) }
    /// ```
    #[instrument(skip(self), fields(game_pk = game_pk))]
    pub async fn get_live_game(
        &self,
        game_pk: u64,
    ) -> Result<LiveGameFeed, MlbApiError> {
        let url = format!(
            "{}/game/{}/feed/live",
            self.live_base_url, game_pk
        );
        self.get_json(&url).await
    }

    // ----------------------------------------------------------------
    // Standings
    // ----------------------------------------------------------------

    /// Returns standings for a given league and season.
    ///
    /// Equivalent to
    /// `GET /api/v1/standings?leagueId={leagueId}&season={season}`.
    ///
    /// `league_id` accepts anything that converts to [`LeagueId`] —
    /// including [`ballpark::League`] variants when the `ballpark` crate
    /// is in use.
    ///
    /// # Example
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use mlb_stats_api::{MlbClient, models::LeagueId};
    /// let client = MlbClient::new();
    ///
    /// // Using raw LeagueId
    /// let standings = client.get_standings(LeagueId(103), 2026).await?;
    ///
    /// // Using ballpark::League (via Into<LeagueId>)
    /// // let standings = client.get_standings(League::AmericanLeague, 2026).await?;
    /// # Ok(()) }
    /// ```
    #[instrument(skip(self), fields(league_id = %league_id.0, season = season))]
    pub async fn get_standings(
        &self,
        league_id: impl Into<LeagueId>,
        season: u32,
    ) -> Result<StandingsResponse, MlbApiError> {
        let league_id = league_id.into();
        let url = format!(
            "{}/standings?leagueId={}&season={}",
            self.base_url, league_id, season
        );
        self.get_json(&url).await
    }

    // ----------------------------------------------------------------
    // Internal helpers
    // ----------------------------------------------------------------

    #[instrument(skip(self))]
    async fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, MlbApiError> {
        tracing::debug!(url = url, "GET");
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
                "HTTP {} for {}",
                response.status(),
                url
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

// ----------------------------------------------------------------
// team_id_from_abbr
//
// Convenience lookup for callers who have a team abbreviation string
// and need a TeamId without taking a dependency on ballpark.
//
// NOTE: This table duplicates data in ballpark::TeamInfo statics.
// That duplication is unavoidable — mlb-stats-api cannot depend on
// ballpark without inverting the dependency graph. When possible,
// prefer ballpark's Team enum and its FromStr impl over this function.
// ----------------------------------------------------------------

/// Returns the [`TeamId`] for a given team abbreviation.
///
/// Matches case-insensitively against conventional box-score
/// abbreviations (e.g. `"TEX"`, `"NYY"`, `"BOS"`).
///
/// When using the `ballpark` crate, prefer parsing via
/// `"TEX".parse::<ballpark::Team>()` and converting with `Into<TeamId>`,
/// which also supports nicknames, city names, and full team names.
///
/// # Errors
///
/// Returns [`MlbApiError::TeamNotFound`] if the abbreviation is not
/// recognised.
///
/// # Example
/// ```rust
/// use mlb_stats_api::{team_id_from_abbr, models::TeamId};
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

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(abbrs.len(), 30, "sanity: 30 abbreviations");
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
        assert_eq!(original_len, ids.len(), "duplicate team IDs in lookup table");
    }
}