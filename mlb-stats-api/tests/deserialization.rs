// mlb-stats-api/tests/deserialization.rs
//
// Fixture-based deserialization tests.
//
// Each test loads a JSON file from tests/fixtures/ and deserializes it into
// the corresponding response type. Tests skip gracefully (with a printed
// notice) when the fixture file does not exist — run
// `bash scripts/fetch_fixtures.sh` from the workspace root to populate them.
//
// IMPORTANT: Fixture files are real API responses saved verbatim. They are
// never hand-crafted. If a test fails, that means either:
//   (a) the model type doesn't match what the API actually returns, or
//   (b) the fixture file is stale (re-run fetch_fixtures.sh to refresh).
//
// Tests do NOT assert on field values beyond basic sanity checks — the
// goal is to verify that deserialization succeeds and that known-stable
// structural invariants hold.

use mlb_stats_api::models::{
    attendance::AttendanceResponse,
    game::{
        BoxscoreResponse, ContextMetrics, GameContent, GamePaceResponse, HighLowResponse,
        LinescoreResponse, WinProbabilityEntry,
    },
    league::{DivisionsResponse, LeagueResponse, SportsResponse},
    live::LiveGameFeed,
    meta::MetaEntry,
    plays::PlayByPlayResponse,
    roster::{PeopleResponse, RosterResponse, TeamsResponse},
    schedule::ScheduleResponse,
    season::SeasonsResponse,
    standings::StandingsResponse,
    stats::{LeagueLeadersResponse, StatsResponse},
    umpires::UmpiresResponse,
    venue::VenuesResponse,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Load a fixture file by name from tests/fixtures/.
/// Returns None if the file doesn't exist — tests must handle this with a skip.
fn load_fixture(name: &str) -> Option<String> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let path = format!("{manifest}/tests/fixtures/{name}");
    match std::fs::read_to_string(&path) {
        Ok(s) => Some(s),
        Err(_) => {
            println!(
                "SKIP: fixture not found at {path}. \
                 Run `bash scripts/fetch_fixtures.sh` to populate."
            );
            None
        }
    }
}

/// Read the final gamePk from .fixtures-meta.json, with a fallback.
fn fixture_game_pk_final() -> u64 {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let path = format!("{manifest}/tests/fixtures/.fixtures-meta.json");
    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
            if let Some(pk) = v["game_pk_final"].as_u64() {
                return pk;
            }
        }
    }
    825024 // fallback: known stable completed game
}

// ---------------------------------------------------------------------------
// Schedule
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_schedule_all_games() {
    let Some(json) = load_fixture("schedule_all_games.json") else { return };
    let resp: ScheduleResponse = serde_json::from_str(&json)
        .expect("schedule_all_games.json should deserialize into ScheduleResponse");
    // A full-day schedule should have at least one date with games, except
    // on off-days. We only assert the type round-trips.
    let _ = resp.total_items;
}

#[test]
fn test_deserialize_schedule_game_today() {
    let Some(json) = load_fixture("schedule_game_today.json") else { return };
    let resp: ScheduleResponse = serde_json::from_str(&json)
        .expect("schedule_game_today.json should deserialize into ScheduleResponse");
    // If there's a game, it should have a gamePk
    if let Some(date) = resp.dates.first() {
        if let Some(game) = date.games.first() {
            assert!(game.game_pk.is_some(), "scheduled game should have a gamePk");
        }
    }
}

#[test]
fn test_deserialize_schedule_no_game() {
    let Some(json) = load_fixture("schedule_no_game.json") else { return };
    let resp: ScheduleResponse = serde_json::from_str(&json)
        .expect("schedule_no_game.json should deserialize into ScheduleResponse");
    // Off-day: dates array is empty
    assert!(
        resp.dates.is_empty(),
        "off-day schedule should have empty dates[], got {} dates",
        resp.dates.len()
    );
}

#[test]
fn test_deserialize_schedule_hydrated() {
    let Some(json) = load_fixture("schedule_game_hydrated.json") else { return };
    let _resp: ScheduleResponse = serde_json::from_str(&json)
        .expect("schedule_game_hydrated.json should deserialize into ScheduleResponse");
    // Hydrated response embeds additional fields — just verifying no panic
}

// ---------------------------------------------------------------------------
// Live game feed
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_live_game_feed_final() {
    let Some(json) = load_fixture("live_game_feed_final.json") else { return };
    let feed: LiveGameFeed = serde_json::from_str(&json)
        .expect("live_game_feed_final.json should deserialize into LiveGameFeed");
    let game_pk = fixture_game_pk_final();
    assert_eq!(
        feed.game_pk,
        Some(game_pk),
        "live feed gamePk should match .fixtures-meta.json game_pk_final"
    );
    // A completed game must have a Final abstract state
    if let Some(ref game_data) = feed.game_data {
        if let Some(ref status) = game_data.status {
            if let Some(ref state) = status.abstract_game_state {
                assert_eq!(state, "Final", "completed game should be in Final state");
            }
        }
    }
}

#[test]
fn test_deserialize_live_game_feed_pregame() {
    let Some(json) = load_fixture("live_game_feed_pregame.json") else { return };
    let feed: LiveGameFeed = serde_json::from_str(&json)
        .expect("live_game_feed_pregame.json should deserialize into LiveGameFeed");
    if let Some(ref game_data) = feed.game_data {
        if let Some(ref status) = game_data.status {
            if let Some(ref state) = status.abstract_game_state {
                assert_eq!(state, "Preview", "pregame feed should be in Preview state");
            }
        }
    }
}

#[test]
fn test_deserialize_live_game_feed_fields() {
    let Some(json) = load_fixture("live_game_feed_fields.json") else { return };
    // A fields-filtered response is a strict subset of LiveGameFeed.
    // All missing fields are Option<T> so they deserialize as None.
    let _feed: LiveGameFeed = serde_json::from_str(&json)
        .expect("live_game_feed_fields.json should deserialize into LiveGameFeed");
}

// ---------------------------------------------------------------------------
// Linescore & Boxscore
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_linescore() {
    let Some(json) = load_fixture("linescore.json") else { return };
    let resp: LinescoreResponse = serde_json::from_str(&json)
        .expect("linescore.json should deserialize into LinescoreResponse");
    if let Some(ref innings) = resp.innings {
        assert!(!innings.is_empty(), "completed game linescore should have innings");
    }
}

#[test]
fn test_deserialize_boxscore() {
    let Some(json) = load_fixture("boxscore.json") else { return };
    let resp: BoxscoreResponse = serde_json::from_str(&json)
        .expect("boxscore.json should deserialize into BoxscoreResponse");
    if let Some(ref teams) = resp.teams {
        assert!(teams.away.is_some(), "boxscore should have away team");
        assert!(teams.home.is_some(), "boxscore should have home team");
    }
}

// ---------------------------------------------------------------------------
// Play-by-play, win probability, content, context metrics
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_play_by_play() {
    let Some(json) = load_fixture("play_by_play.json") else { return };
    let resp: PlayByPlayResponse = serde_json::from_str(&json)
        .expect("play_by_play.json should deserialize into PlayByPlayResponse");
    assert!(
        !resp.all_plays.is_empty(),
        "play-by-play for completed game should have plays"
    );
}

#[test]
fn test_deserialize_win_probability() {
    let Some(json) = load_fixture("win_probability.json") else { return };
    let entries: Vec<WinProbabilityEntry> = serde_json::from_str(&json)
        .expect("win_probability.json should deserialize into Vec<WinProbabilityEntry>");
    assert!(
        !entries.is_empty(),
        "win probability for completed game should have entries"
    );
}

#[test]
fn test_deserialize_game_content() {
    let Some(json) = load_fixture("game_content.json") else { return };
    let _content: GameContent = serde_json::from_str(&json)
        .expect("game_content.json should deserialize into GameContent");
}

#[test]
fn test_deserialize_context_metrics() {
    let Some(json) = load_fixture("context_metrics.json") else { return };
    let _metrics: ContextMetrics = serde_json::from_str(&json)
        .expect("context_metrics.json should deserialize into ContextMetrics");
}

// ---------------------------------------------------------------------------
// Standings
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_standings() {
    let Some(json) = load_fixture("standings.json") else { return };
    let resp: StandingsResponse = serde_json::from_str(&json)
        .expect("standings.json should deserialize into StandingsResponse");
    // Both leagues requested — 6 divisions total
    assert_eq!(
        resp.records.len(),
        6,
        "combined AL+NL standings should have 6 division records"
    );
    for record in &resp.records {
        assert_eq!(
            record.team_records.len(),
            5,
            "each division should have 5 team records"
        );
    }
}

// ---------------------------------------------------------------------------
// Roster & People
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_roster_active() {
    let Some(json) = load_fixture("roster_active.json") else { return };
    let resp: RosterResponse = serde_json::from_str(&json)
        .expect("roster_active.json should deserialize into RosterResponse");
    let count = resp.roster.len();
    assert!(
        (20..=40).contains(&count),
        "active roster should have 20–40 players, got {count}"
    );
}

#[test]
fn test_deserialize_roster_40man() {
    let Some(json) = load_fixture("roster_40man.json") else { return };
    let resp: RosterResponse = serde_json::from_str(&json)
        .expect("roster_40man.json should deserialize into RosterResponse");
    let count = resp.roster.len();
    assert!(
        (30..=50).contains(&count),
        "40-man roster should have 30–50 players, got {count}"
    );
}

#[test]
fn test_deserialize_player_bio() {
    let Some(json) = load_fixture("player_bio.json") else { return };
    let resp: PeopleResponse = serde_json::from_str(&json)
        .expect("player_bio.json should deserialize into PeopleResponse");
    assert!(
        !resp.people.is_empty(),
        "player_bio.json should have at least one person"
    );
}

#[test]
fn test_deserialize_player_stats_batter() {
    let Some(json) = load_fixture("player_stats_batter.json") else { return };
    let resp: PeopleResponse = serde_json::from_str(&json)
        .expect("player_stats_batter.json should deserialize into PeopleResponse");
    // Mike Trout's ID is stable
    let person = resp.people.first().expect("should have a person");
    assert_eq!(person.id, Some(545361), "player_stats_batter.json should be Trout (545361)");
}

#[test]
fn test_deserialize_player_stats_pitcher() {
    let Some(json) = load_fixture("player_stats_pitcher.json") else { return };
    let resp: PeopleResponse = serde_json::from_str(&json)
        .expect("player_stats_pitcher.json should deserialize into PeopleResponse");
    let person = resp.people.first().expect("should have a person");
    assert_eq!(person.id, Some(543243), "player_stats_pitcher.json should be Eovaldi (543243)");
}

#[test]
fn test_deserialize_teams() {
    let Some(json) = load_fixture("teams.json") else { return };
    let resp: TeamsResponse = serde_json::from_str(&json)
        .expect("teams.json should deserialize into TeamsResponse");
    assert_eq!(resp.teams.len(), 30, "MLB should have 30 teams");
}

// ---------------------------------------------------------------------------
// Venue
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_venue() {
    let Some(json) = load_fixture("venue.json") else { return };
    let resp: VenuesResponse = serde_json::from_str(&json)
        .expect("venue.json should deserialize into VenuesResponse");
    let venue = resp.venues.first().expect("should have at least one venue");
    assert_eq!(venue.id, Some(5325), "venue.json should be Globe Life Field (5325)");
}

// ---------------------------------------------------------------------------
// League / Division / Sport
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_leagues() {
    let Some(json) = load_fixture("leagues.json") else { return };
    let resp: LeagueResponse = serde_json::from_str(&json)
        .expect("leagues.json should deserialize into LeagueResponse");
    assert_eq!(resp.leagues.len(), 2, "sportId=1 should return 2 leagues (AL + NL)");
}

#[test]
fn test_deserialize_divisions() {
    let Some(json) = load_fixture("divisions.json") else { return };
    let resp: DivisionsResponse = serde_json::from_str(&json)
        .expect("divisions.json should deserialize into DivisionsResponse");
    assert_eq!(resp.divisions.len(), 6, "MLB has 6 divisions");
}

#[test]
fn test_deserialize_sports() {
    let Some(json) = load_fixture("sports.json") else { return };
    let _resp: SportsResponse = serde_json::from_str(&json)
        .expect("sports.json should deserialize into SportsResponse");
}

// ---------------------------------------------------------------------------
// Season
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_season() {
    let Some(json) = load_fixture("season.json") else { return };
    let resp: SeasonsResponse = serde_json::from_str(&json)
        .expect("season.json should deserialize into SeasonsResponse");
    assert!(!resp.seasons.is_empty(), "season.json should have at least one season");
}

#[test]
fn test_deserialize_seasons_all() {
    let Some(json) = load_fixture("seasons_all.json") else { return };
    let resp: SeasonsResponse = serde_json::from_str(&json)
        .expect("seasons_all.json should deserialize into SeasonsResponse");
    // MLB has seasons going back to 1876 — well over 100
    assert!(
        resp.seasons.len() > 100,
        "all seasons should have >100 entries, got {}",
        resp.seasons.len()
    );
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_stats_hitting() {
    let Some(json) = load_fixture("stats_hitting_season.json") else { return };
    let resp: StatsResponse = serde_json::from_str(&json)
        .expect("stats_hitting_season.json should deserialize into StatsResponse");
    assert!(!resp.stats.is_empty(), "hitting stats should have at least one stat group");
}

#[test]
fn test_deserialize_league_leaders() {
    let Some(json) = load_fixture("stats_hr_leaders.json") else { return };
    let resp: LeagueLeadersResponse = serde_json::from_str(&json)
        .expect("stats_hr_leaders.json should deserialize into LeagueLeadersResponse");
    assert!(!resp.league_leaders.is_empty(), "HR leaders should have at least one category");
}

// ---------------------------------------------------------------------------
// Attendance
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_attendance() {
    let Some(json) = load_fixture("attendance.json") else { return };
    // Shape is uncertain (records field is Value) — just verifying no panic
    let _resp: AttendanceResponse = serde_json::from_str(&json)
        .expect("attendance.json should deserialize into AttendanceResponse");
}

// ---------------------------------------------------------------------------
// Umpires / Officials
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_officials() {
    let Some(json) = load_fixture("officials.json") else { return };
    let _resp: UmpiresResponse = serde_json::from_str(&json)
        .expect("officials.json should deserialize into UmpiresResponse");
}

// ---------------------------------------------------------------------------
// Meta
// ---------------------------------------------------------------------------

#[test]
fn test_deserialize_meta_game_types() {
    let Some(json) = load_fixture("meta_game_types.json") else { return };
    let entries: Vec<MetaEntry> = serde_json::from_str(&json)
        .expect("meta_game_types.json should deserialize into Vec<MetaEntry>");
    let codes: Vec<_> = entries.iter().filter_map(|e| e.code.as_deref()).collect();
    assert!(codes.contains(&"R"), "gameTypes should include R (regular season)");
    assert!(codes.contains(&"W"), "gameTypes should include W (World Series)");
}

#[test]
fn test_deserialize_meta_pitch_types() {
    let Some(json) = load_fixture("meta_pitch_types.json") else { return };
    let entries: Vec<MetaEntry> = serde_json::from_str(&json)
        .expect("meta_pitch_types.json should deserialize into Vec<MetaEntry>");
    let codes: Vec<_> = entries.iter().filter_map(|e| e.code.as_deref()).collect();
    assert!(codes.contains(&"FF"), "pitchTypes should include FF (four-seam fastball)");
}

// ---------------------------------------------------------------------------
// Statcast / Pitch data sanity
// ---------------------------------------------------------------------------

#[test]
fn test_statcast_pitch_speed_plausible() {
    let Some(json) = load_fixture("live_game_feed_final.json") else { return };
    let feed: LiveGameFeed = serde_json::from_str(&json).unwrap();
    let live_data = match feed.live_data { Some(ld) => ld, None => return };
    let plays = match live_data.plays { Some(p) => p, None => return };

    // Find the first pitch with Statcast speed data
    let pitch = plays.all_plays.iter()
        .flat_map(|play| &play.play_events)
        .find(|ev| ev.pitch_data.as_ref().and_then(|pd| pd.start_speed.as_ref()).is_some());

    if let Some(ev) = pitch {
        let speed = ev.pitch_data.as_ref().unwrap().start_speed.unwrap();
        assert!(
            (50.0..=110.0).contains(&speed),
            "pitch start_speed {speed:.1} mph is outside plausible range 50–110"
        );
    }
}

// ---------------------------------------------------------------------------
// gamePk consistency across endpoints
// ---------------------------------------------------------------------------

#[test]
fn test_game_pk_consistent_across_fixtures() {
    let game_pk = fixture_game_pk_final();

    for fixture_name in &["live_game_feed_final.json", "linescore.json"] {
        let Some(json) = load_fixture(fixture_name) else { continue };
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        if let Some(pk) = v.get("gamePk").and_then(|x| x.as_u64()) {
            assert_eq!(
                pk, game_pk,
                "{fixture_name}: gamePk {pk} should match .fixtures-meta.json game_pk_final {game_pk}"
            );
        }
    }
}