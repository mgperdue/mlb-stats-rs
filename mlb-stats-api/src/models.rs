//! Response model types for the MLB Stats API.
//!
//! Models are organized by endpoint domain. All types use `Option<T>` for
//! most fields because the API is undocumented and field presence varies
//! by endpoint, game state, and hydration parameters.
//!
//! # `serde_json::Value` exceptions
//!
//! Two patterns in the API cannot be represented as typed structs and use
//! `serde_json::Value` explicitly:
//!
//! 1. **Player maps** keyed by `"ID{playerId}"` strings (e.g. `"ID592450"`).
//!    Appears in `GameData::players` and `BoxscoreTeam::players`. Parse
//!    individual entries with `serde_json::from_value::<PlayerGameEntry>(v)`.
//!
//! 2. **`allPlays` array** in `Plays::all_plays`. This array can exceed 200
//!    entries in a finished game. Use the `fields` query parameter when only
//!    current play state is needed.
//!
//! Every other use of `Value` in these models represents a field whose
//! schema is either undocumented or highly variable in practice.

pub mod common;
pub mod live;
pub mod meta;
pub mod roster;
pub mod schedule;
pub mod standings;

// Re-export the most commonly used types at the models level for ergonomic
// access. Consumers can use `mlb_stats_api::models::LiveGameFeed` rather
// than the full path.
pub use common::{
    Broadcast, Decisions, GameStatus, GeoCoordinates, HandSide, LeagueRef, Official, PersonRef,
    Position, SeriesStatus, SportRef, TeamRef, TimeZone, Venue, VenueLocation, VenueRef, Weather,
    WinLossRecord,
};
pub use live::{
    BattingStats, BoxscoreInfoItem, BoxscoreTeam, BoxscoreTeams, Count, Defense,
    FieldingStats, GameData, GameDatetime, GameFlags, GameInfo, GameInfoDetail, GameTeam,
    GameTeams, GameWeather, HitCoordinates, HitData, InningLine, InningScore, Linescore,
    LinescoreTeams, LinescoreTotals, LiveData, LiveGameFeed, Matchup, Offense,
    PitchBreaks, PitchCall, PitchCoordinates, PitchData, PitchingStats, PitchTypeInfo,
    Play, PlayAbout, PlayByInning, PlayEvent, PlayEventDetails, PlayResult, PlayerGameEntry,
    Plays, ProbablePitchers, ReviewInfo, RunnerCredit, RunnerDetails, RunnerMovement,
};
pub use roster::{
    PeopleResponse, Player, RosterEntry, RosterResponse, RosterStatus, StatContainer,
    StatGroup, StatSplit, StatType, Team, TeamsResponse,
};
pub use schedule::{
    GameInfo as ScheduleGameInfo, ScheduleDate, ScheduleLeagueRecord, ScheduleResponse,
    ScheduledGame, ScheduledGameTeam, ScheduledGameTeams,
};
pub use standings::{
    DivisionRecord, LeagueRecord, SplitRecord, Streak, StandingsRecord, StandingsResponse,
    TeamRecordSplits, TeamStandingsRecord,
};
pub use meta::{MetaEntry, MetaType};