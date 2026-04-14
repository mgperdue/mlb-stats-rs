//! Common types shared across MLB Stats API response models.
//!
//! # ID newtypes
//!
//! [`LeagueId`], [`DivisionId`], and [`TeamId`] are strongly-typed
//! wrappers around raw `u32` identifiers. They prevent accidentally
//! passing a team ID where a league ID is expected at compile time,
//! while remaining completely neutral about which values are "valid" —
//! that opinion belongs in `ballpark`.
//!
//! Use the named enums in `ballpark` for ergonomic access:
//!
//! ```rust
//! // mlb-stats-api caller — typed but numeric
//! use mlb_stats_api::models::{LeagueId, TeamId};
//! client.get_standings(LeagueId(103), 2026).await?;
//!
//! // ballpark caller — named constants, no magic numbers
//! use ballpark::{League, Team};
//! client.get_standings(League::AmericanLeague, 2026).await?;
//! client.get_schedule_for_date(Team::Rangers, "2026-04-14").await?;
//! ```
//!
//! # Response reference stubs
//!
//! [`LeagueRef`], [`DivisionRef`], and [`TeamRef`] are the thin embedded
//! objects that appear inline within API responses — typically containing
//! only `id`, `link`, and `name`. They are not full resource
//! representations. Use the dedicated endpoints (`/api/v1/league`,
//! `/api/v1/divisions`, `/api/v1/teams`) for complete data.

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================
// ID newtypes
// ============================================================

/// A strongly-typed league identifier.
///
/// Use [`ballpark::League`] for named constants instead of constructing
/// this directly (e.g. `League::AmericanLeague` instead of `LeagueId(103)`).
///
/// # Example
/// ```rust
/// use mlb_stats_api::models::LeagueId;
///
/// let id = LeagueId(103); // American League
/// let id = LeagueId(104); // National League
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LeagueId(pub u32);

/// A strongly-typed division identifier.
///
/// Use [`ballpark::Division`] for named constants instead of constructing
/// this directly (e.g. `Division::AlEast` instead of `DivisionId(201)`).
///
/// Note that the division IDs are non-sequential and non-obvious —
/// AL West is 200, NL West is 203. This is a key reason to prefer
/// the named enum over raw IDs.
///
/// # Example
/// ```rust
/// use mlb_stats_api::models::DivisionId;
///
/// let id = DivisionId(201); // AL East
/// let id = DivisionId(200); // AL West (not 203!)
/// let id = DivisionId(203); // NL West (not 200!)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DivisionId(pub u32);

/// A strongly-typed team identifier.
///
/// Use [`ballpark::Team`] for named constants instead of constructing
/// this directly (e.g. `Team::Rangers` instead of `TeamId(140)`).
///
/// # Example
/// ```rust
/// use mlb_stats_api::models::TeamId;
///
/// let id = TeamId(140); // Texas Rangers
/// let id = TeamId(147); // New York Yankees
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamId(pub u32);

impl fmt::Display for LeagueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for DivisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================
// Response reference stubs
//
// These are the thin embedded objects that appear inline within API
// responses. They are NOT full resource representations — they contain
// only enough fields to identify the referenced entity. Use the
// dedicated endpoints for complete data:
//   /api/v1/league        → full league data
//   /api/v1/divisions     → full division data
//   /api/v1/teams/{id}    → full team data
// ============================================================

/// A league reference as embedded in API responses.
///
/// This is a thin stub — not a full league representation. Fields other
/// than `id` are frequently absent depending on the endpoint and hydration
/// level. Use `/api/v1/league` for complete league data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueRef {
    /// The league identifier. Use [`ballpark::League::from`] to convert
    /// to a named enum variant.
    pub id: Option<u32>,
    pub name: Option<String>,
    pub link: Option<String>,
}

/// A division reference as embedded in API responses.
///
/// This is a thin stub — not a full division representation. Fields other
/// than `id` are frequently absent depending on the endpoint and hydration
/// level. Use `/api/v1/divisions` for complete division data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DivisionRef {
    /// The division identifier. Use [`ballpark::Division::from`] to convert
    /// to a named enum variant.
    pub id: Option<u32>,
    pub name: Option<String>,
    pub link: Option<String>,
}

/// A team reference as embedded in API responses.
///
/// This is a thin stub — not a full team representation. Fields other
/// than `id` are frequently absent depending on the endpoint and hydration
/// level. Use `/api/v1/teams/{teamId}` for complete team data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRef {
    /// The team identifier. Use [`ballpark::Team::from`] to convert
    /// to a named enum variant.
    pub id: Option<u32>,
    pub name: Option<String>,
    pub link: Option<String>,
}

// ============================================================
// Other common response types
// ============================================================

/// A venue reference as embedded in API responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VenueRef {
    pub id: Option<u32>,
    pub name: Option<String>,
    pub link: Option<String>,
}

/// A player reference as embedded in API responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRef {
    pub id: Option<u32>,
    pub full_name: Option<String>,
    pub link: Option<String>,
}

/// Batting or throwing handedness.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandSide {
    pub code: Option<String>,
    pub description: Option<String>,
}

/// A win-loss record as embedded in API responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeagueRecord {
    pub wins: Option<u32>,
    pub losses: Option<u32>,
    pub ties: Option<u32>,
    pub pct: Option<String>,
}

/// The status of a game.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStatus {
    /// Broad state: `"Preview"`, `"Live"`, or `"Final"`.
    pub abstract_game_state: Option<String>,
    /// Single character code: `"I"` = In Progress, `"F"` = Final, etc.
    pub coded_game_state: Option<String>,
    /// Human-readable detail: `"In Progress"`, `"Delayed: Rain"`, etc.
    pub detailed_state: Option<String>,
    /// Expanded status code: `"IR"` = In Progress + Rain Delay, etc.
    pub status_code: Option<String>,
    pub start_time_tbd: Option<bool>,
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn league_id_display() {
        assert_eq!(LeagueId(103).to_string(), "103");
    }

    #[test]
    fn division_id_display() {
        assert_eq!(DivisionId(201).to_string(), "201");
    }

    #[test]
    fn team_id_display() {
        assert_eq!(TeamId(140).to_string(), "140");
    }

    #[test]
    fn league_id_copy() {
        let a = LeagueId(103);
        let b = a; // Copy
        assert_eq!(a, b);
    }

    #[test]
    fn league_ref_deserializes_partial() {
        // The API often returns stubs with only id and link populated
        let json = r#"{"id": 103, "link": "/api/v1/league/103"}"#;
        let r: LeagueRef = serde_json::from_str(json).unwrap();
        assert_eq!(r.id, Some(103));
        assert!(r.name.is_none());
    }

    #[test]
    fn team_ref_deserializes_partial() {
        let json = r#"{"id": 140, "name": "Texas Rangers", "link": "/api/v1/teams/140"}"#;
        let r: TeamRef = serde_json::from_str(json).unwrap();
        assert_eq!(r.id, Some(140));
        assert_eq!(r.name.as_deref(), Some("Texas Rangers"));
    }
}