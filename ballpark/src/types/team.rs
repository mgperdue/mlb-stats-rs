//! MLB organizational hierarchy: leagues, divisions, and teams.
//!
//! This module provides named enums for all three levels of MLB's
//! organizational structure, along with associated static metadata and
//! ergonomic string parsing.
//!
//! # Design
//!
//! Each level has two types:
//!
//! - An **enum** (`League`, `Division`, `Team`) — a lightweight, `Copy`able
//!   handle used in function signatures, match arms, and stored in structs.
//!   Includes an `Unknown(u32)` catch-all for unrecognised API values.
//!
//! - An **info struct** (`LeagueInfo`, `DivisionInfo`, `TeamInfo`) — static
//!   descriptive data accessed via the enum's `.info()` method. Never
//!   constructed directly by callers.
//!
//! # Hierarchy
//!
//! The organizational hierarchy is navigable in both directions:
//!
//! ```rust
//! use ballpark::{League, Division, Team};
//!
//! // Top-down
//! let divisions = League::NationalLeague.info().unwrap().divisions;
//! let teams     = Division::NlCentral.info().unwrap().teams;
//!
//! // Bottom-up
//! let division = Team::Cubs.division();
//! let league   = Team::Cubs.league();
//! ```
//!
//! # String parsing
//!
//! All three enums implement [`std::str::FromStr`], accepting a variety
//! of natural inputs case-insensitively:
//!
//! ```rust
//! use ballpark::{League, Division, Team};
//!
//! let league: League   = "AL".parse().unwrap();
//! let league: League   = "American League".parse().unwrap();
//! let div: Division    = "ALE".parse().unwrap();
//! let div: Division    = "AL East".parse().unwrap();
//! let team: Team       = "TEX".parse().unwrap();
//! let team: Team       = "Rangers".parse().unwrap();
//! let team: Team       = "Texas".parse().unwrap();
//! let team: Team       = "Texas Rangers".parse().unwrap();
//! ```
//!
//! # ID conversion
//!
//! All three enums convert to and from the [`LeagueId`], [`DivisionId`],
//! and [`TeamId`] newtypes in `mlb-stats-api`:
//!
//! ```rust
//! use ballpark::Team;
//! use mlb_stats_api::models::TeamId;
//!
//! let id: TeamId = Team::Rangers.into();   // TeamId(140)
//! let team = Team::from(TeamId(140));       // Team::Rangers
//! ```
//!
//! # Stability
//!
//! The static metadata in this file reflects the MLB organizational
//! structure at the time of publication. Division realignments and team
//! relocations are extremely rare (the last realignment was 1994), but
//! when they occur a crate update will be required. For guaranteed-current
//! team names and IDs, prefer the live `get_teams()` API response over
//! the statics — see `ballpark::Client::fetch_team_registry()`.

use std::fmt;
use mlb_stats_api::models::{DivisionId, LeagueId, TeamId};

// ============================================================
// LeagueInfo
// ============================================================

/// Static metadata for an MLB league.
///
/// Obtain via [`League::info`] — do not construct directly.
#[derive(Debug, PartialEq, Eq)]
pub struct LeagueInfo {
    /// MLB Stats API league identifier.
    pub id: u32,
    /// Full league name — e.g. `"American League"`.
    pub name: &'static str,
    /// Short name — e.g. `"American"`.
    pub short_name: &'static str,
    /// Conventional two-letter abbreviation — e.g. `"AL"`.
    pub abbreviation: &'static str,
    /// The three divisions belonging to this league.
    pub divisions: &'static [Division],
}

static AL: LeagueInfo = LeagueInfo {
    id: 103,
    name: "American League",
    short_name: "American",
    abbreviation: "AL",
    divisions: &[Division::AlEast, Division::AlCentral, Division::AlWest],
};

static NL: LeagueInfo = LeagueInfo {
    id: 104,
    name: "National League",
    short_name: "National",
    abbreviation: "NL",
    divisions: &[Division::NlEast, Division::NlCentral, Division::NlWest],
};

// ============================================================
// League
// ============================================================

/// The two MLB leagues.
///
/// `Unknown` exists to handle any future structural change without
/// panicking on an unrecognised API value.
///
/// # Example
/// ```rust
/// use ballpark::League;
/// use mlb_stats_api::models::LeagueId;
///
/// let info = League::AmericanLeague.info().unwrap();
/// assert_eq!(info.abbreviation, "AL");
/// assert_eq!(info.divisions.len(), 3);
///
/// let id: LeagueId = League::NationalLeague.into();
/// assert_eq!(id, LeagueId(104));
///
/// let back = League::from(LeagueId(103));
/// assert_eq!(back, League::AmericanLeague);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum League {
    /// The American League (ID 103).
    AmericanLeague,
    /// The National League (ID 104).
    NationalLeague,
    /// An unrecognised league ID returned by the API.
    Unknown(u32),
}

impl League {
    /// Returns static metadata for this league, or `None` for [`League::Unknown`].
    ///
    /// # Example
    /// ```rust
    /// use ballpark::League;
    ///
    /// let info = League::AmericanLeague.info().unwrap();
    /// assert_eq!(info.name, "American League");
    /// assert_eq!(info.short_name, "American");
    /// assert_eq!(info.abbreviation, "AL");
    /// assert_eq!(info.divisions.len(), 3);
    ///
    /// assert!(League::Unknown(999).info().is_none());
    /// ```
    pub fn info(self) -> Option<&'static LeagueInfo> {
        match self {
            League::AmericanLeague => Some(&AL),
            League::NationalLeague => Some(&NL),
            League::Unknown(_) => None,
        }
    }

    /// Returns the MLB Stats API identifier for this league.
    ///
    /// # Example
    /// ```rust
    /// use ballpark::League;
    ///
    /// assert_eq!(League::AmericanLeague.id().0, 103);
    /// assert_eq!(League::NationalLeague.id().0, 104);
    /// ```
    pub fn id(self) -> LeagueId {
        self.into()
    }
}

impl From<League> for LeagueId {
    fn from(league: League) -> LeagueId {
        LeagueId(match league {
            League::AmericanLeague => 103,
            League::NationalLeague => 104,
            League::Unknown(id) => id,
        })
    }
}

impl From<LeagueId> for League {
    fn from(id: LeagueId) -> League {
        match id.0 {
            103 => League::AmericanLeague,
            104 => League::NationalLeague,
            other => League::Unknown(other),
        }
    }
}

impl fmt::Display for League {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.info() {
            Some(info) => write!(f, "{}", info.name),
            None => {
                let League::Unknown(id) = self else { unreachable!() };
                write!(f, "Unknown League ({id})")
            }
        }
    }
}

// ============================================================
// DivisionInfo
// ============================================================

/// Static metadata for an MLB division.
///
/// Obtain via [`Division::info`] — do not construct directly.
#[derive(Debug, PartialEq, Eq)]
pub struct DivisionInfo {
    /// MLB Stats API division identifier.
    pub id: u32,
    /// Full division name — e.g. `"AL East"`.
    pub name: &'static str,
    /// Conventional three-letter abbreviation — e.g. `"ALE"`.
    pub abbreviation: &'static str,
    /// The league this division belongs to.
    pub league: League,
    /// The five teams in this division.
    pub teams: &'static [Team],
}

// AL divisions.
//
// IMPORTANT: AL West is ID 200, not 203. NL West is 203, not 200.
// This non-obvious swap is the key footgun these enums exist to prevent.
static AL_EAST: DivisionInfo = DivisionInfo {
    id: 201,
    name: "AL East",
    abbreviation: "ALE",
    league: League::AmericanLeague,
    teams: &[Team::Orioles, Team::RedSox, Team::Yankees, Team::Rays, Team::BlueJays],
};

static AL_CENTRAL: DivisionInfo = DivisionInfo {
    id: 202,
    name: "AL Central",
    abbreviation: "ALC",
    league: League::AmericanLeague,
    teams: &[Team::WhiteSox, Team::Guardians, Team::Tigers, Team::Royals, Team::Twins],
};

static AL_WEST: DivisionInfo = DivisionInfo {
    id: 200,
    name: "AL West",
    abbreviation: "ALW",
    league: League::AmericanLeague,
    teams: &[Team::Astros, Team::Angels, Team::Athletics, Team::Mariners, Team::Rangers],
};

// NL divisions.
static NL_EAST: DivisionInfo = DivisionInfo {
    id: 204,
    name: "NL East",
    abbreviation: "NLE",
    league: League::NationalLeague,
    teams: &[Team::Braves, Team::Marlins, Team::Mets, Team::Phillies, Team::Nationals],
};

static NL_CENTRAL: DivisionInfo = DivisionInfo {
    id: 205,
    name: "NL Central",
    abbreviation: "NLC",
    league: League::NationalLeague,
    teams: &[Team::Cubs, Team::Reds, Team::Brewers, Team::Pirates, Team::Cardinals],
};

static NL_WEST: DivisionInfo = DivisionInfo {
    id: 203,
    name: "NL West",
    abbreviation: "NLW",
    league: League::NationalLeague,
    teams: &[Team::Diamondbacks, Team::Rockies, Team::Dodgers, Team::Padres, Team::Giants],
};

// ============================================================
// Division
// ============================================================

/// The six MLB divisions.
///
/// `Unknown` exists to handle any future structural change without
/// panicking on an unrecognised API value.
///
/// # Example
/// ```rust
/// use ballpark::{Division, League};
///
/// let info = Division::NlCentral.info().unwrap();
/// assert_eq!(info.league, League::NationalLeague);
/// assert_eq!(info.teams.len(), 5);
///
/// // AL West is ID 200, NL West is ID 203 — the enum hides this footgun.
/// assert_eq!(Division::AlWest.id().0, 200);
/// assert_eq!(Division::NlWest.id().0, 203);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Division {
    /// AL East (ID 201)
    AlEast,
    /// AL Central (ID 202)
    AlCentral,
    /// AL West (ID 200) — note: not 203
    AlWest,
    /// NL East (ID 204)
    NlEast,
    /// NL Central (ID 205)
    NlCentral,
    /// NL West (ID 203) — note: not 200
    NlWest,
    /// An unrecognised division ID returned by the API.
    Unknown(u32),
}

impl Division {
    /// Returns static metadata for this division, or `None` for [`Division::Unknown`].
    ///
    /// # Example
    /// ```rust
    /// use ballpark::Division;
    ///
    /// let info = Division::AlEast.info().unwrap();
    /// assert_eq!(info.name, "AL East");
    /// assert_eq!(info.abbreviation, "ALE");
    /// assert_eq!(info.teams.len(), 5);
    ///
    /// assert!(Division::Unknown(999).info().is_none());
    /// ```
    pub fn info(self) -> Option<&'static DivisionInfo> {
        match self {
            Division::AlEast    => Some(&AL_EAST),
            Division::AlCentral => Some(&AL_CENTRAL),
            Division::AlWest    => Some(&AL_WEST),
            Division::NlEast    => Some(&NL_EAST),
            Division::NlCentral => Some(&NL_CENTRAL),
            Division::NlWest    => Some(&NL_WEST),
            Division::Unknown(_) => None,
        }
    }

    /// Returns the MLB Stats API identifier for this division.
    ///
    /// # Example
    /// ```rust
    /// use ballpark::Division;
    ///
    /// assert_eq!(Division::AlWest.id().0, 200);
    /// assert_eq!(Division::NlWest.id().0, 203);
    /// ```
    pub fn id(self) -> DivisionId {
        self.into()
    }

    /// Returns the league this division belongs to.
    ///
    /// Returns [`League::Unknown`]`(0)` for [`Division::Unknown`].
    ///
    /// # Example
    /// ```rust
    /// use ballpark::{Division, League};
    ///
    /// assert_eq!(Division::NlEast.league(), League::NationalLeague);
    /// assert_eq!(Division::AlCentral.league(), League::AmericanLeague);
    /// ```
    pub fn league(self) -> League {
        match self.info() {
            Some(info) => info.league,
            None => League::Unknown(0),
        }
    }
}

impl From<Division> for DivisionId {
    fn from(division: Division) -> DivisionId {
        DivisionId(match division {
            Division::AlEast    => 201,
            Division::AlCentral => 202,
            Division::AlWest    => 200,
            Division::NlEast    => 204,
            Division::NlCentral => 205,
            Division::NlWest    => 203,
            Division::Unknown(id) => id,
        })
    }
}

impl From<DivisionId> for Division {
    fn from(id: DivisionId) -> Division {
        match id.0 {
            200 => Division::AlWest,
            201 => Division::AlEast,
            202 => Division::AlCentral,
            203 => Division::NlWest,
            204 => Division::NlEast,
            205 => Division::NlCentral,
            other => Division::Unknown(other),
        }
    }
}

impl fmt::Display for Division {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.info() {
            Some(info) => write!(f, "{}", info.name),
            None => {
                let Division::Unknown(id) = self else { unreachable!() };
                write!(f, "Unknown Division ({id})")
            }
        }
    }
}

// ============================================================
// TeamInfo
// ============================================================

/// Static metadata for an MLB team.
///
/// Obtain via [`Team::info`] — do not construct directly.
#[derive(Debug, PartialEq, Eq)]
pub struct TeamInfo {
    /// MLB Stats API team identifier.
    pub id: u32,
    /// Full team name including city — e.g. `"Boston Red Sox"`.
    pub name: &'static str,
    /// City or region — e.g. `"Boston"`, `"Texas"`, `"San Francisco"`.
    ///
    /// Empty string for franchises with no city prefix (currently only
    /// the Athletics, who dropped their city name after relocating).
    pub location: &'static str,
    /// Team nickname — e.g. `"Red Sox"`, `"Rangers"`, `"Giants"`.
    pub nickname: &'static str,
    /// Conventional box-score abbreviation — e.g. `"BOS"`, `"TEX"`, `"SF"`.
    pub abbreviation: &'static str,
    /// The division this team belongs to.
    pub division: Division,
}

// AL East
static ORIOLES:   TeamInfo = TeamInfo { id: 110, name: "Baltimore Orioles",     location: "Baltimore",      nickname: "Orioles",      abbreviation: "BAL", division: Division::AlEast };
static RED_SOX:   TeamInfo = TeamInfo { id: 111, name: "Boston Red Sox",         location: "Boston",         nickname: "Red Sox",      abbreviation: "BOS", division: Division::AlEast };
static YANKEES:   TeamInfo = TeamInfo { id: 147, name: "New York Yankees",       location: "New York",       nickname: "Yankees",      abbreviation: "NYY", division: Division::AlEast };
static RAYS:      TeamInfo = TeamInfo { id: 139, name: "Tampa Bay Rays",         location: "Tampa Bay",      nickname: "Rays",         abbreviation: "TB",  division: Division::AlEast };
static BLUE_JAYS: TeamInfo = TeamInfo { id: 141, name: "Toronto Blue Jays",      location: "Toronto",        nickname: "Blue Jays",    abbreviation: "TOR", division: Division::AlEast };

// AL Central
static WHITE_SOX:  TeamInfo = TeamInfo { id: 145, name: "Chicago White Sox",    location: "Chicago",        nickname: "White Sox",    abbreviation: "CWS", division: Division::AlCentral };
static GUARDIANS:  TeamInfo = TeamInfo { id: 114, name: "Cleveland Guardians",  location: "Cleveland",      nickname: "Guardians",    abbreviation: "CLE", division: Division::AlCentral };
static TIGERS:     TeamInfo = TeamInfo { id: 116, name: "Detroit Tigers",       location: "Detroit",        nickname: "Tigers",       abbreviation: "DET", division: Division::AlCentral };
static ROYALS:     TeamInfo = TeamInfo { id: 118, name: "Kansas City Royals",   location: "Kansas City",    nickname: "Royals",       abbreviation: "KC",  division: Division::AlCentral };
static TWINS:      TeamInfo = TeamInfo { id: 142, name: "Minnesota Twins",      location: "Minnesota",      nickname: "Twins",        abbreviation: "MIN", division: Division::AlCentral };

// AL West
//
// Athletics: location is empty — the franchise dropped its city name after
// relocating to Sacramento in 2025 and has not yet adopted a new city prefix
// pending the permanent move to Las Vegas. Parsing "Athletics" works via the
// nickname match; location-based lookup is intentionally unsupported.
static ASTROS:    TeamInfo = TeamInfo { id: 117, name: "Houston Astros",        location: "Houston",        nickname: "Astros",       abbreviation: "HOU", division: Division::AlWest };
static ANGELS:    TeamInfo = TeamInfo { id: 108, name: "Los Angeles Angels",    location: "Los Angeles",    nickname: "Angels",       abbreviation: "LAA", division: Division::AlWest };
static ATHLETICS: TeamInfo = TeamInfo { id: 133, name: "Athletics",             location: "",               nickname: "Athletics",    abbreviation: "ATH", division: Division::AlWest };
static MARINERS:  TeamInfo = TeamInfo { id: 136, name: "Seattle Mariners",      location: "Seattle",        nickname: "Mariners",     abbreviation: "SEA", division: Division::AlWest };
static RANGERS:   TeamInfo = TeamInfo { id: 140, name: "Texas Rangers",         location: "Texas",          nickname: "Rangers",      abbreviation: "TEX", division: Division::AlWest };

// NL East
static BRAVES:    TeamInfo = TeamInfo { id: 144, name: "Atlanta Braves",        location: "Atlanta",        nickname: "Braves",       abbreviation: "ATL", division: Division::NlEast };
static MARLINS:   TeamInfo = TeamInfo { id: 146, name: "Miami Marlins",         location: "Miami",          nickname: "Marlins",      abbreviation: "MIA", division: Division::NlEast };
static METS:      TeamInfo = TeamInfo { id: 121, name: "New York Mets",         location: "New York",       nickname: "Mets",         abbreviation: "NYM", division: Division::NlEast };
static PHILLIES:  TeamInfo = TeamInfo { id: 143, name: "Philadelphia Phillies", location: "Philadelphia",   nickname: "Phillies",     abbreviation: "PHI", division: Division::NlEast };
static NATIONALS: TeamInfo = TeamInfo { id: 120, name: "Washington Nationals",  location: "Washington",     nickname: "Nationals",    abbreviation: "WSH", division: Division::NlEast };

// NL Central
static CUBS:      TeamInfo = TeamInfo { id: 112, name: "Chicago Cubs",          location: "Chicago",        nickname: "Cubs",         abbreviation: "CHC", division: Division::NlCentral };
static REDS:      TeamInfo = TeamInfo { id: 113, name: "Cincinnati Reds",       location: "Cincinnati",     nickname: "Reds",         abbreviation: "CIN", division: Division::NlCentral };
static BREWERS:   TeamInfo = TeamInfo { id: 158, name: "Milwaukee Brewers",     location: "Milwaukee",      nickname: "Brewers",      abbreviation: "MIL", division: Division::NlCentral };
static PIRATES:   TeamInfo = TeamInfo { id: 134, name: "Pittsburgh Pirates",    location: "Pittsburgh",     nickname: "Pirates",      abbreviation: "PIT", division: Division::NlCentral };
static CARDINALS: TeamInfo = TeamInfo { id: 138, name: "St. Louis Cardinals",   location: "St. Louis",      nickname: "Cardinals",    abbreviation: "STL", division: Division::NlCentral };

// NL West
static DIAMONDBACKS: TeamInfo = TeamInfo { id: 109, name: "Arizona Diamondbacks",  location: "Arizona",       nickname: "Diamondbacks", abbreviation: "ARI", division: Division::NlWest };
static ROCKIES:      TeamInfo = TeamInfo { id: 115, name: "Colorado Rockies",      location: "Colorado",      nickname: "Rockies",      abbreviation: "COL", division: Division::NlWest };
static DODGERS:      TeamInfo = TeamInfo { id: 119, name: "Los Angeles Dodgers",   location: "Los Angeles",   nickname: "Dodgers",      abbreviation: "LAD", division: Division::NlWest };
static PADRES:       TeamInfo = TeamInfo { id: 135, name: "San Diego Padres",      location: "San Diego",     nickname: "Padres",       abbreviation: "SD",  division: Division::NlWest };
static GIANTS:       TeamInfo = TeamInfo { id: 137, name: "San Francisco Giants",  location: "San Francisco", nickname: "Giants",       abbreviation: "SF",  division: Division::NlWest };

// ============================================================
// Team
// ============================================================

/// All 30 current MLB franchises.
///
/// Variant names use the team's nickname rather than city to remain
/// stable across relocations. `Unknown` handles any ID not in this
/// list — historical franchises, future expansion, or API anomalies.
///
/// # Example
/// ```rust
/// use ballpark::{Team, Division, League};
/// use mlb_stats_api::models::TeamId;
///
/// let info = Team::Yankees.info().unwrap();
/// assert_eq!(info.name, "New York Yankees");
/// assert_eq!(info.abbreviation, "NYY");
///
/// assert_eq!(Team::Cubs.division(), Division::NlCentral);
/// assert_eq!(Team::Cubs.league(), League::NationalLeague);
///
/// let id: TeamId = Team::Rangers.into();
/// assert_eq!(id, TeamId(140));
///
/// let team = Team::from(TeamId(140));
/// assert_eq!(team, Team::Rangers);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Team {
    // AL East
    /// Baltimore Orioles (ID 110)
    Orioles,
    /// Boston Red Sox (ID 111)
    RedSox,
    /// New York Yankees (ID 147)
    Yankees,
    /// Tampa Bay Rays (ID 139)
    Rays,
    /// Toronto Blue Jays (ID 141)
    BlueJays,

    // AL Central
    /// Chicago White Sox (ID 145)
    WhiteSox,
    /// Cleveland Guardians (ID 114)
    Guardians,
    /// Detroit Tigers (ID 116)
    Tigers,
    /// Kansas City Royals (ID 118)
    Royals,
    /// Minnesota Twins (ID 142)
    Twins,

    // AL West
    /// Houston Astros (ID 117)
    Astros,
    /// Los Angeles Angels (ID 108)
    Angels,
    /// Athletics (ID 133) — Sacramento 2025+, eventually Las Vegas
    Athletics,
    /// Seattle Mariners (ID 136)
    Mariners,
    /// Texas Rangers (ID 140)
    Rangers,

    // NL East
    /// Atlanta Braves (ID 144)
    Braves,
    /// Miami Marlins (ID 146)
    Marlins,
    /// New York Mets (ID 121)
    Mets,
    /// Philadelphia Phillies (ID 143)
    Phillies,
    /// Washington Nationals (ID 120)
    Nationals,

    // NL Central
    /// Chicago Cubs (ID 112)
    Cubs,
    /// Cincinnati Reds (ID 113)
    Reds,
    /// Milwaukee Brewers (ID 158)
    Brewers,
    /// Pittsburgh Pirates (ID 134)
    Pirates,
    /// St. Louis Cardinals (ID 138)
    Cardinals,

    // NL West
    /// Arizona Diamondbacks (ID 109)
    Diamondbacks,
    /// Colorado Rockies (ID 115)
    Rockies,
    /// Los Angeles Dodgers (ID 119)
    Dodgers,
    /// San Diego Padres (ID 135)
    Padres,
    /// San Francisco Giants (ID 137)
    Giants,

    /// An unrecognised team ID returned by the API.
    ///
    /// May represent a historical franchise, an expansion team, or a
    /// data anomaly. The inner value is the raw API team ID.
    Unknown(u32),
}

impl Team {
    /// Returns static metadata for this team, or `None` for [`Team::Unknown`].
    ///
    /// # Example
    /// ```rust
    /// use ballpark::Team;
    ///
    /// let info = Team::Rangers.info().unwrap();
    /// assert_eq!(info.name, "Texas Rangers");
    /// assert_eq!(info.location, "Texas");
    /// assert_eq!(info.nickname, "Rangers");
    /// assert_eq!(info.abbreviation, "TEX");
    ///
    /// assert!(Team::Unknown(9999).info().is_none());
    /// ```
    pub fn info(self) -> Option<&'static TeamInfo> {
        match self {
            Team::Orioles      => Some(&ORIOLES),
            Team::RedSox       => Some(&RED_SOX),
            Team::Yankees      => Some(&YANKEES),
            Team::Rays         => Some(&RAYS),
            Team::BlueJays     => Some(&BLUE_JAYS),
            Team::WhiteSox     => Some(&WHITE_SOX),
            Team::Guardians    => Some(&GUARDIANS),
            Team::Tigers       => Some(&TIGERS),
            Team::Royals       => Some(&ROYALS),
            Team::Twins        => Some(&TWINS),
            Team::Astros       => Some(&ASTROS),
            Team::Angels       => Some(&ANGELS),
            Team::Athletics    => Some(&ATHLETICS),
            Team::Mariners     => Some(&MARINERS),
            Team::Rangers      => Some(&RANGERS),
            Team::Braves       => Some(&BRAVES),
            Team::Marlins      => Some(&MARLINS),
            Team::Mets         => Some(&METS),
            Team::Phillies     => Some(&PHILLIES),
            Team::Nationals    => Some(&NATIONALS),
            Team::Cubs         => Some(&CUBS),
            Team::Reds         => Some(&REDS),
            Team::Brewers      => Some(&BREWERS),
            Team::Pirates      => Some(&PIRATES),
            Team::Cardinals    => Some(&CARDINALS),
            Team::Diamondbacks => Some(&DIAMONDBACKS),
            Team::Rockies      => Some(&ROCKIES),
            Team::Dodgers      => Some(&DODGERS),
            Team::Padres       => Some(&PADRES),
            Team::Giants       => Some(&GIANTS),
            Team::Unknown(_)   => None,
        }
    }

    /// Returns the MLB Stats API identifier for this team.
    ///
    /// # Example
    /// ```rust
    /// use ballpark::Team;
    ///
    /// assert_eq!(Team::Rangers.id().0, 140);
    /// ```
    pub fn id(self) -> TeamId {
        self.into()
    }

    /// Returns the division this team belongs to.
    ///
    /// Returns [`Division::Unknown`]`(0)` for [`Team::Unknown`].
    ///
    /// # Example
    /// ```rust
    /// use ballpark::{Team, Division};
    ///
    /// assert_eq!(Team::Cubs.division(), Division::NlCentral);
    /// assert_eq!(Team::Rangers.division(), Division::AlWest);
    /// ```
    pub fn division(self) -> Division {
        match self.info() {
            Some(info) => info.division,
            None => Division::Unknown(0),
        }
    }

    /// Returns the league this team belongs to.
    ///
    /// Delegates to [`Team::division`] then [`Division::league`].
    ///
    /// # Example
    /// ```rust
    /// use ballpark::{Team, League};
    ///
    /// assert_eq!(Team::Cardinals.league(), League::NationalLeague);
    /// assert_eq!(Team::Rangers.league(), League::AmericanLeague);
    /// ```
    pub fn league(self) -> League {
        self.division().league()
    }
}

impl From<Team> for TeamId {
    fn from(team: Team) -> TeamId {
        TeamId(match team {
            Team::Orioles      => 110,
            Team::RedSox       => 111,
            Team::Yankees      => 147,
            Team::Rays         => 139,
            Team::BlueJays     => 141,
            Team::WhiteSox     => 145,
            Team::Guardians    => 114,
            Team::Tigers       => 116,
            Team::Royals       => 118,
            Team::Twins        => 142,
            Team::Astros       => 117,
            Team::Angels       => 108,
            Team::Athletics    => 133,
            Team::Mariners     => 136,
            Team::Rangers      => 140,
            Team::Braves       => 144,
            Team::Marlins      => 146,
            Team::Mets         => 121,
            Team::Phillies     => 143,
            Team::Nationals    => 120,
            Team::Cubs         => 112,
            Team::Reds         => 113,
            Team::Brewers      => 158,
            Team::Pirates      => 134,
            Team::Cardinals    => 138,
            Team::Diamondbacks => 109,
            Team::Rockies      => 115,
            Team::Dodgers      => 119,
            Team::Padres       => 135,
            Team::Giants       => 137,
            Team::Unknown(id)  => id,
        })
    }
}

impl From<TeamId> for Team {
    fn from(id: TeamId) -> Team {
        match id.0 {
            110 => Team::Orioles,
            111 => Team::RedSox,
            147 => Team::Yankees,
            139 => Team::Rays,
            141 => Team::BlueJays,
            145 => Team::WhiteSox,
            114 => Team::Guardians,
            116 => Team::Tigers,
            118 => Team::Royals,
            142 => Team::Twins,
            117 => Team::Astros,
            108 => Team::Angels,
            133 => Team::Athletics,
            136 => Team::Mariners,
            140 => Team::Rangers,
            144 => Team::Braves,
            146 => Team::Marlins,
            121 => Team::Mets,
            143 => Team::Phillies,
            120 => Team::Nationals,
            112 => Team::Cubs,
            113 => Team::Reds,
            158 => Team::Brewers,
            134 => Team::Pirates,
            138 => Team::Cardinals,
            109 => Team::Diamondbacks,
            115 => Team::Rockies,
            119 => Team::Dodgers,
            135 => Team::Padres,
            137 => Team::Giants,
            other => Team::Unknown(other),
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.info() {
            Some(info) => write!(f, "{}", info.name),
            None => {
                let Team::Unknown(id) = self else { unreachable!() };
                write!(f, "Unknown Team ({id})")
            }
        }
    }
}

// ============================================================
// Parse errors
// ============================================================

/// Error returned when a string cannot be matched to a known [`League`].
///
/// # Example
/// ```rust
/// use ballpark::League;
///
/// let err = "MLB".parse::<League>().unwrap_err();
/// assert!(err.to_string().contains("MLB"));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownLeagueInput(String);

impl fmt::Display for UnknownLeagueInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown league: {:?} — try \"AL\", \"NL\", \
             \"American\", \"National\", \"American League\", \
             or \"National League\"",
            self.0
        )
    }
}

impl std::error::Error for UnknownLeagueInput {}

/// Error returned when a string cannot be matched to a known [`Division`].
///
/// # Example
/// ```rust
/// use ballpark::Division;
///
/// let err = "East".parse::<Division>().unwrap_err();
/// assert!(err.to_string().contains("East"));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownDivisionInput(String);

impl fmt::Display for UnknownDivisionInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown division: {:?} — try an abbreviation (\"ALE\") \
             or full name (\"AL East\")",
            self.0
        )
    }
}

impl std::error::Error for UnknownDivisionInput {}

/// Error returned when a string cannot be matched to a known [`Team`].
///
/// # Example
/// ```rust
/// use ballpark::{Team, UnknownTeamInput};
///
/// // Unrecognised input
/// let err = "XYZ".parse::<Team>().unwrap_err();
/// assert!(matches!(err, UnknownTeamInput::NotFound(_)));
///
/// // Ambiguous location
/// let err = "New York".parse::<Team>().unwrap_err();
/// assert!(matches!(err, UnknownTeamInput::Ambiguous { .. }));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum UnknownTeamInput {
    /// No team matched the input.
    NotFound(String),
    /// The input matched more than one team.
    ///
    /// For example, `"New York"` matches both the Yankees and the Mets,
    /// and `"Los Angeles"` matches both the Angels and the Dodgers.
    Ambiguous {
        input: String,
        matches: Vec<Team>,
    },
}

impl fmt::Display for UnknownTeamInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnknownTeamInput::NotFound(s) => write!(
                f,
                "unknown team: {:?} — try an abbreviation (\"TEX\"), \
                 nickname (\"Rangers\"), location (\"Texas\"), \
                 or full name (\"Texas Rangers\")",
                s
            ),
            UnknownTeamInput::Ambiguous { input, matches } => {
                let names: Vec<&str> = matches
                    .iter()
                    .filter_map(|t| t.info().map(|i| i.name))
                    .collect();
                write!(
                    f,
                    "ambiguous team: {:?} matches {} — \
                     please be more specific",
                    input,
                    names.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for UnknownTeamInput {}

// ============================================================
// FromStr implementations
// ============================================================

impl std::str::FromStr for League {
    type Err = UnknownLeagueInput;

    /// Parses a league name or abbreviation (case-insensitive) into a [`League`].
    ///
    /// Accepted inputs:
    /// - Abbreviation: `"AL"`, `"NL"`
    /// - Short name: `"American"`, `"National"`
    /// - Full name: `"American League"`, `"National League"`
    ///
    /// # Example
    /// ```rust
    /// use ballpark::League;
    ///
    /// assert_eq!("AL".parse::<League>().unwrap(),              League::AmericanLeague);
    /// assert_eq!("American".parse::<League>().unwrap(),        League::AmericanLeague);
    /// assert_eq!("American League".parse::<League>().unwrap(), League::AmericanLeague);
    /// assert_eq!("national league".parse::<League>().unwrap(), League::NationalLeague);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_uppercase();
        all_known_leagues()
            .into_iter()
            .find(|league| {
                league.info()
                    .map(|info| {
                        info.abbreviation.to_ascii_uppercase() == normalized
                            || info.short_name.to_ascii_uppercase() == normalized
                            || info.name.to_ascii_uppercase() == normalized
                    })
                    .unwrap_or(false)
            })
            .ok_or_else(|| UnknownLeagueInput(s.to_string()))
    }
}

impl std::str::FromStr for Division {
    type Err = UnknownDivisionInput;

    /// Parses a division name or abbreviation (case-insensitive) into a [`Division`].
    ///
    /// Accepted inputs:
    /// - Abbreviation: `"ALE"`, `"ALC"`, `"ALW"`, `"NLE"`, `"NLC"`, `"NLW"`
    /// - Full name: `"AL East"`, `"NL Central"`, etc.
    ///
    /// # Example
    /// ```rust
    /// use ballpark::Division;
    ///
    /// assert_eq!("ALE".parse::<Division>().unwrap(),     Division::AlEast);
    /// assert_eq!("al east".parse::<Division>().unwrap(), Division::AlEast);
    /// assert_eq!("NL West".parse::<Division>().unwrap(), Division::NlWest);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_uppercase();
        all_known_divisions()
            .into_iter()
            .find(|div| {
                div.info()
                    .map(|info| {
                        info.abbreviation.to_ascii_uppercase() == normalized
                            || info.name.to_ascii_uppercase() == normalized
                    })
                    .unwrap_or(false)
            })
            .ok_or_else(|| UnknownDivisionInput(s.to_string()))
    }
}

impl std::str::FromStr for Team {
    type Err = UnknownTeamInput;

    /// Parses a team name, nickname, location, or abbreviation
    /// (case-insensitive) into a [`Team`].
    ///
    /// Accepted inputs:
    /// - Abbreviation: `"TEX"`, `"NYY"`, `"BOS"`
    /// - Nickname: `"Rangers"`, `"Yankees"`, `"Red Sox"`
    /// - Location: `"Texas"`, `"Boston"` (unambiguous locations only)
    /// - Full name: `"Texas Rangers"`, `"New York Yankees"`, `"Boston Red Sox"`
    ///
    /// Returns [`UnknownTeamInput::Ambiguous`] when the input matches more
    /// than one team — for example `"New York"` matches both the Yankees
    /// and the Mets, and `"Los Angeles"` matches both the Angels and the
    /// Dodgers. Use a nickname or full name to disambiguate.
    ///
    /// # Example
    /// ```rust
    /// use ballpark::Team;
    ///
    /// assert_eq!("TEX".parse::<Team>().unwrap(),           Team::Rangers);
    /// assert_eq!("Rangers".parse::<Team>().unwrap(),       Team::Rangers);
    /// assert_eq!("Texas".parse::<Team>().unwrap(),         Team::Rangers);
    /// assert_eq!("Texas Rangers".parse::<Team>().unwrap(), Team::Rangers);
    /// assert_eq!("texas rangers".parse::<Team>().unwrap(), Team::Rangers);
    ///
    /// // Ambiguous — use nickname or full name instead
    /// assert!("New York".parse::<Team>().is_err());
    /// assert_eq!("Yankees".parse::<Team>().unwrap(), Team::Yankees);
    /// assert_eq!("Mets".parse::<Team>().unwrap(),    Team::Mets);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_uppercase();
        let matches: Vec<Team> = all_known_teams()
            .into_iter()
            .filter(|team| {
                team.info()
                    .map(|info| {
                        info.abbreviation.to_ascii_uppercase() == normalized
                            || info.nickname.to_ascii_uppercase() == normalized
                            || info.name.to_ascii_uppercase() == normalized
                            || (!info.location.is_empty()
                                && info.location.to_ascii_uppercase() == normalized)
                    })
                    .unwrap_or(false)
            })
            .collect();

        match matches.len() {
            0 => Err(UnknownTeamInput::NotFound(s.to_string())),
            1 => Ok(matches[0]),
            _ => Err(UnknownTeamInput::Ambiguous {
                input: s.to_string(),
                matches,
            }),
        }
    }
}

// ============================================================
// Private helpers
// ============================================================

fn all_known_leagues() -> Vec<League> {
    vec![League::AmericanLeague, League::NationalLeague]
}

fn all_known_divisions() -> Vec<Division> {
    use Division::*;
    vec![AlEast, AlCentral, AlWest, NlEast, NlCentral, NlWest]
}

fn all_known_teams() -> Vec<Team> {
    use Team::*;
    vec![
        Orioles, RedSox, Yankees, Rays, BlueJays,
        WhiteSox, Guardians, Tigers, Royals, Twins,
        Astros, Angels, Athletics, Mariners, Rangers,
        Braves, Marlins, Mets, Phillies, Nationals,
        Cubs, Reds, Brewers, Pirates, Cardinals,
        Diamondbacks, Rockies, Dodgers, Padres, Giants,
    ]
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Round-trips ----

    #[test]
    fn league_round_trips() {
        for league in [League::AmericanLeague, League::NationalLeague] {
            let id: LeagueId = league.into();
            let back = League::from(id);
            assert_eq!(league, back, "round-trip failed for {league}");
        }
    }

    #[test]
    fn division_round_trips() {
        for div in all_known_divisions() {
            let id: DivisionId = div.into();
            let back = Division::from(id);
            assert_eq!(div, back, "round-trip failed for {div}");
        }
    }

    #[test]
    fn all_30_teams_round_trip() {
        let teams = all_known_teams();
        assert_eq!(teams.len(), 30, "sanity: exactly 30 known teams");
        for team in teams {
            let id: TeamId = team.into();
            let back = Team::from(id);
            assert_eq!(team, back, "round-trip failed for {team}");
        }
    }

    // ---- Known ID footguns ----

    #[test]
    fn al_west_id_is_200_not_203() {
        assert_eq!(DivisionId::from(Division::AlWest).0, 200);
        assert_eq!(DivisionId::from(Division::NlWest).0, 203);
    }

    #[test]
    fn brewers_id_is_158() {
        // Breaks the otherwise sequential run of 133–147
        assert_eq!(TeamId::from(Team::Brewers).0, 158);
    }

    // ---- Unknown variants ----

    #[test]
    fn unknown_variants_round_trip() {
        let team = Team::from(TeamId(9999));
        assert_eq!(team, Team::Unknown(9999));
        assert_eq!(TeamId::from(team).0, 9999);

        let div = Division::from(DivisionId(999));
        assert_eq!(div, Division::Unknown(999));

        let league = League::from(LeagueId(999));
        assert_eq!(league, League::Unknown(999));
    }

    #[test]
    fn unknown_variants_return_none_for_info() {
        assert!(Team::Unknown(9999).info().is_none());
        assert!(Division::Unknown(999).info().is_none());
        assert!(League::Unknown(999).info().is_none());
    }

    // ---- Structural integrity ----

    #[test]
    fn division_league_membership() {
        for div in [Division::AlEast, Division::AlCentral, Division::AlWest] {
            assert_eq!(div.league(), League::AmericanLeague, "{div} should be AL");
        }
        for div in [Division::NlEast, Division::NlCentral, Division::NlWest] {
            assert_eq!(div.league(), League::NationalLeague, "{div} should be NL");
        }
    }

    #[test]
    fn each_division_has_exactly_5_teams() {
        for div in all_known_divisions() {
            assert_eq!(
                div.info().unwrap().teams.len(), 5,
                "{div} should have exactly 5 teams"
            );
        }
    }

    #[test]
    fn each_league_has_exactly_3_divisions() {
        for league in [League::AmericanLeague, League::NationalLeague] {
            assert_eq!(
                league.info().unwrap().divisions.len(), 3,
                "{league} should have exactly 3 divisions"
            );
        }
    }

    #[test]
    fn team_division_and_league_are_consistent() {
        for team in all_known_teams() {
            assert_eq!(
                team.league(),
                team.division().league(),
                "{team}: league() and division().league() disagree"
            );
        }
    }

    #[test]
    fn all_teams_appear_in_their_division() {
        for team in all_known_teams() {
            let div = team.division();
            assert!(
                div.info().unwrap().teams.contains(&team),
                "{team} not found in {div}.info().teams"
            );
        }
    }

    #[test]
    fn all_divisions_appear_in_their_league() {
        for div in all_known_divisions() {
            let league = div.league();
            assert!(
                league.info().unwrap().divisions.contains(&div),
                "{div} not found in {league}.info().divisions"
            );
        }
    }

    #[test]
    fn team_ids_are_unique() {
        let mut ids: Vec<u32> = all_known_teams()
            .iter()
            .map(|t| TeamId::from(*t).0)
            .collect();
        let original_len = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(original_len, ids.len(), "duplicate team IDs found");
    }

    #[test]
    fn division_ids_are_unique() {
        let mut ids: Vec<u32> = all_known_divisions()
            .iter()
            .map(|d| DivisionId::from(*d).0)
            .collect();
        let original_len = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(original_len, ids.len(), "duplicate division IDs found");
    }

    #[test]
    fn display_uses_full_name() {
        assert_eq!(Team::Yankees.to_string(),          "New York Yankees");
        assert_eq!(Team::Athletics.to_string(),         "Athletics");
        assert_eq!(Division::NlCentral.to_string(),    "NL Central");
        assert_eq!(League::NationalLeague.to_string(), "National League");
        assert_eq!(Team::Unknown(9999).to_string(),    "Unknown Team (9999)");
        assert_eq!(Division::Unknown(99).to_string(),  "Unknown Division (99)");
        assert_eq!(League::Unknown(9).to_string(),     "Unknown League (9)");
    }

    // ---- League parsing ----

    #[test]
    fn parse_league_abbreviation() {
        assert_eq!("AL".parse::<League>().unwrap(), League::AmericanLeague);
        assert_eq!("NL".parse::<League>().unwrap(), League::NationalLeague);
    }

    #[test]
    fn parse_league_short_name() {
        assert_eq!("American".parse::<League>().unwrap(), League::AmericanLeague);
        assert_eq!("National".parse::<League>().unwrap(), League::NationalLeague);
    }

    #[test]
    fn parse_league_full_name() {
        assert_eq!("American League".parse::<League>().unwrap(), League::AmericanLeague);
        assert_eq!("National League".parse::<League>().unwrap(), League::NationalLeague);
    }

    #[test]
    fn parse_league_case_insensitive() {
        assert_eq!("al".parse::<League>().unwrap(),              League::AmericanLeague);
        assert_eq!("american".parse::<League>().unwrap(),        League::AmericanLeague);
        assert_eq!("NATIONAL LEAGUE".parse::<League>().unwrap(), League::NationalLeague);
    }

    #[test]
    fn parse_league_trims_whitespace() {
        assert_eq!("  AL  ".parse::<League>().unwrap(), League::AmericanLeague);
    }

    #[test]
    fn parse_league_unknown_returns_err() {
        assert!("MLB".parse::<League>().is_err());
        assert!("".parse::<League>().is_err());
    }

    // ---- Division parsing ----

    #[test]
    fn parse_division_abbreviation() {
        assert_eq!("ALE".parse::<Division>().unwrap(), Division::AlEast);
        assert_eq!("NLW".parse::<Division>().unwrap(), Division::NlWest);
    }

    #[test]
    fn parse_division_full_name() {
        assert_eq!("AL East".parse::<Division>().unwrap(),    Division::AlEast);
        assert_eq!("NL Central".parse::<Division>().unwrap(), Division::NlCentral);
    }

    #[test]
    fn parse_division_case_insensitive() {
        assert_eq!("al east".parse::<Division>().unwrap(), Division::AlEast);
        assert_eq!("AL EAST".parse::<Division>().unwrap(), Division::AlEast);
        assert_eq!("nlw".parse::<Division>().unwrap(),     Division::NlWest);
    }

    #[test]
    fn parse_division_trims_whitespace() {
        assert_eq!("  ALE  ".parse::<Division>().unwrap(), Division::AlEast);
    }

    #[test]
    fn parse_all_6_division_abbreviations() {
        for div in all_known_divisions() {
            let abbr = div.info().unwrap().abbreviation;
            let parsed: Division = abbr.parse()
                .unwrap_or_else(|_| panic!("{abbr} should parse"));
            assert_eq!(parsed, div, "parse round-trip failed for {abbr}");
        }
    }

    #[test]
    fn parse_all_6_division_full_names() {
        for div in all_known_divisions() {
            let name = div.info().unwrap().name;
            let parsed: Division = name.parse()
                .unwrap_or_else(|_| panic!("{name} should parse"));
            assert_eq!(parsed, div, "parse round-trip failed for {name}");
        }
    }

    #[test]
    fn parse_division_unknown_returns_err() {
        assert!("AL".parse::<Division>().is_err());
        assert!("East".parse::<Division>().is_err());
        assert!("".parse::<Division>().is_err());
    }

    // ---- Team parsing ----

    #[test]
    fn parse_team_abbreviation() {
        assert_eq!("TEX".parse::<Team>().unwrap(), Team::Rangers);
        assert_eq!("NYY".parse::<Team>().unwrap(), Team::Yankees);
        assert_eq!("BOS".parse::<Team>().unwrap(), Team::RedSox);
    }

    #[test]
    fn parse_team_nickname() {
        assert_eq!("Rangers".parse::<Team>().unwrap(),   Team::Rangers);
        assert_eq!("Yankees".parse::<Team>().unwrap(),   Team::Yankees);
        assert_eq!("Red Sox".parse::<Team>().unwrap(),   Team::RedSox);
        assert_eq!("Blue Jays".parse::<Team>().unwrap(), Team::BlueJays);
        assert_eq!("Athletics".parse::<Team>().unwrap(), Team::Athletics);
    }

    #[test]
    fn parse_team_location_unambiguous() {
        assert_eq!("Texas".parse::<Team>().unwrap(),   Team::Rangers);
        assert_eq!("Boston".parse::<Team>().unwrap(),  Team::RedSox);
        assert_eq!("Houston".parse::<Team>().unwrap(), Team::Astros);
        assert_eq!("Toronto".parse::<Team>().unwrap(), Team::BlueJays);
    }

    #[test]
    fn parse_team_full_name() {
        assert_eq!("Texas Rangers".parse::<Team>().unwrap(),     Team::Rangers);
        assert_eq!("New York Yankees".parse::<Team>().unwrap(),  Team::Yankees);
        assert_eq!("Boston Red Sox".parse::<Team>().unwrap(),    Team::RedSox);
        assert_eq!("St. Louis Cardinals".parse::<Team>().unwrap(), Team::Cardinals);
    }

    #[test]
    fn parse_team_case_insensitive() {
        assert_eq!("texas rangers".parse::<Team>().unwrap(), Team::Rangers);
        assert_eq!("YANKEES".parse::<Team>().unwrap(),       Team::Yankees);
        assert_eq!("red sox".parse::<Team>().unwrap(),       Team::RedSox);
        assert_eq!("tex".parse::<Team>().unwrap(),           Team::Rangers);
    }

    #[test]
    fn parse_team_trims_whitespace() {
        assert_eq!("  TEX  ".parse::<Team>().unwrap(), Team::Rangers);
    }

    #[test]
    fn parse_team_ambiguous_location_returns_err() {
        // "New York" matches Yankees and Mets
        let err = "New York".parse::<Team>().unwrap_err();
        assert!(matches!(err, UnknownTeamInput::Ambiguous { .. }));
        if let UnknownTeamInput::Ambiguous { matches, .. } = err {
            assert!(matches.contains(&Team::Yankees));
            assert!(matches.contains(&Team::Mets));
        }

        // "Los Angeles" matches Angels and Dodgers
        let err = "Los Angeles".parse::<Team>().unwrap_err();
        assert!(matches!(err, UnknownTeamInput::Ambiguous { .. }));
        if let UnknownTeamInput::Ambiguous { matches, .. } = err {
            assert!(matches.contains(&Team::Angels));
            assert!(matches.contains(&Team::Dodgers));
        }

        // "Chicago" matches Cubs and White Sox
        let err = "Chicago".parse::<Team>().unwrap_err();
        assert!(matches!(err, UnknownTeamInput::Ambiguous { .. }));
        if let UnknownTeamInput::Ambiguous { matches, .. } = err {
            assert!(matches.contains(&Team::Cubs));
            assert!(matches.contains(&Team::WhiteSox));
        }
    }

    #[test]
    fn parse_team_not_found_returns_err() {
        let err = "XYZ".parse::<Team>().unwrap_err();
        assert!(matches!(err, UnknownTeamInput::NotFound(_)));
        assert!("".parse::<Team>().is_err());
    }

    #[test]
    fn parse_all_30_abbreviations() {
        for team in all_known_teams() {
            let abbr = team.info().unwrap().abbreviation;
            let parsed: Team = abbr.parse()
                .unwrap_or_else(|_| panic!("{abbr} should parse"));
            assert_eq!(parsed, team, "abbreviation parse round-trip failed for {abbr}");
        }
    }

    #[test]
    fn parse_all_30_nicknames() {
        for team in all_known_teams() {
            let nickname = team.info().unwrap().nickname;
            let parsed: Team = nickname.parse()
                .unwrap_or_else(|_| panic!("{nickname} should parse"));
            assert_eq!(parsed, team, "nickname parse round-trip failed for {nickname}");
        }
    }

    #[test]
    fn parse_all_30_full_names() {
        for team in all_known_teams() {
            let name = team.info().unwrap().name;
            let parsed: Team = name.parse()
                .unwrap_or_else(|_| panic!("{name} should parse"));
            assert_eq!(parsed, team, "full name parse round-trip failed for {name}");
        }
    }

    #[test]
    fn ambiguous_error_message_names_conflicting_teams() {
        let err = "New York".parse::<Team>().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("New York Yankees"), "error should name Yankees: {msg}");
        assert!(msg.contains("New York Mets"),    "error should name Mets: {msg}");
    }

    #[test]
    fn not_found_error_message_includes_input() {
        let err = "XYZ".parse::<Team>().unwrap_err();
        assert!(err.to_string().contains("XYZ"));
    }
}