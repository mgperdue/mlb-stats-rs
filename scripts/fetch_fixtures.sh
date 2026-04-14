#!/usr/bin/env bash
# fetch_fixtures.sh
#
# Fetches real MLB Stats API responses and saves them as fixture files for
# use in deserialization unit tests. Run this from the workspace root:
#
#   bash scripts/fetch_fixtures.sh
#
# Fixtures are saved verbatim — no modifications, no formatting. They represent
# exactly what the API returned at the time of the fetch. All fixture files land
# flat in mlb-stats-api/tests/fixtures/ — no subdirectories.
#
# Requirements: curl, python3
#
# This script is NOT run in CI. It is run manually by maintainers to refresh
# fixtures, and the resulting files are committed to the repository.
#
# Game anchors used for game-specific fixtures:
#
#   GAME_PK_FINAL = 825024
#     A known completed regular-season game used for all game-specific
#     endpoints (linescore, boxscore, play-by-play, win probability, officials,
#     context metrics, content). Update this comment once the fixture has been
#     fetched and the game is confirmed.
#
#   GAME_PK_PREGAME
#     Extracted dynamically from today's Rangers schedule. Used for the
#     pregame live feed fixture. Skipped gracefully if no game is scheduled.
#
# To override the final gamePk without editing this file:
#   GAME_PK_FINAL=<pk> bash scripts/fetch_fixtures.sh

set -euo pipefail

BASE="https://statsapi.mlb.com/api/v1"
BASE_V11="https://statsapi.mlb.com/api/v1.1"
FIXTURE_DIR="mlb-stats-api/tests/fixtures"

GAME_PK_FINAL="${GAME_PK_FINAL:-825024}"
TEAM_ID=140               # Texas Rangers
DATE_TODAY=$(date +"%m/%d/%Y")   # MM/DD/YYYY as required by the schedule endpoint
SEASON=$(date +"%Y")
AL_LEAGUE_ID=103
NL_LEAGUE_ID=104
TROUT_PLAYER_ID=545361    # Mike Trout — long-career player with stable ID
EOVALDI_ID=543243         # Nathan Eovaldi — Rangers SP with stable ID
GLOBE_LIFE_VENUE_ID=5325  # Globe Life Field, Arlington TX

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

die() { echo "ERROR: $1" >&2; exit 1; }

command -v curl    >/dev/null || die "curl is required"
command -v python3 >/dev/null || die "python3 is required"

fetch() {
    local url="$1"
    local outfile="$2"
    echo "  GET $url"
    curl --silent --fail --show-error \
         --header "Accept: application/json" \
         "$url" \
        | python3 -m json.tool --indent 2 \
        > "$FIXTURE_DIR/$outfile" \
        || die "Failed to fetch or parse $url"
    echo "      -> $FIXTURE_DIR/$outfile"
}

mkdir -p "$FIXTURE_DIR"

echo "=== Fetching MLB Stats API fixtures ==="
echo "    GAME_PK_FINAL = $GAME_PK_FINAL"
echo "    SEASON        = $SEASON"
echo "    DATE_TODAY    = $DATE_TODAY"
echo ""

# ---------------------------------------------------------------------------
# Schedule
# ---------------------------------------------------------------------------
echo "==> schedule"

# All games today across all teams
fetch \
    "$BASE/schedule?sportId=1&date=$DATE_TODAY" \
    "schedule_all_games.json"

# Rangers game today (pregame shape — good for testing pre-game schedule)
fetch \
    "$BASE/schedule?sportId=1&date=$DATE_TODAY&teamId=$TEAM_ID" \
    "schedule_game_today.json"

# Rangers game on the final-game date (shape correlation with live feed fixture).
# Resolve the date from the game feed rather than hardcoding it so this script
# stays correct if GAME_PK_FINAL is ever updated.
FINAL_DATE=$(curl --silent --fail \
    "$BASE_V11/game/$GAME_PK_FINAL/feed/live?fields=gameData,datetime,officialDate" \
    | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('gameData',{}).get('datetime',{}).get('officialDate',''))" \
    2>/dev/null || echo "")

if [ -n "$FINAL_DATE" ]; then
    FINAL_DATE_API=$(python3 -c "
from datetime import datetime
print(datetime.strptime('$FINAL_DATE', '%Y-%m-%d').strftime('%m/%d/%Y'))
")
    fetch \
        "$BASE/schedule?sportId=1&date=$FINAL_DATE_API&teamId=$TEAM_ID" \
        "schedule_game_final_date.json"
else
    echo "  Skipping schedule_game_final_date.json (could not resolve date for gamePk $GAME_PK_FINAL)"
fi

# Rangers off-day: March 27 2026 — confirmed rest day between the March 26
# opener in Philadelphia and the next series. Returns a valid ScheduleResponse
# with an empty dates[] array, which exercises the "no games" code path.
fetch \
    "$BASE/schedule?sportId=1&date=03/27/2026&teamId=$TEAM_ID" \
    "schedule_no_game.json"

# Hydrated schedule — shape with linescore + decisions + weather + broadcasts
# + officials all embedded inline
fetch \
    "$BASE/schedule?sportId=1&date=$DATE_TODAY&teamId=$TEAM_ID&hydrate=linescore,decisions,weather,broadcasts,officials" \
    "schedule_game_hydrated.json"

# ---------------------------------------------------------------------------
# Live game feed
# ---------------------------------------------------------------------------
echo ""
echo "==> live game feed"

# Completed game — all fields populated, stable shape
fetch \
    "$BASE_V11/game/$GAME_PK_FINAL/feed/live" \
    "live_game_feed_final.json"

# Fields-filtered subset — verifies that partial deserialization works
# (missing fields must be Option<T> on the model, not required)
fetch \
    "$BASE_V11/game/$GAME_PK_FINAL/feed/live?fields=gamePk,gameData,status,abstractGameState,detailedState" \
    "live_game_feed_fields.json"

# Pregame feed — extracted dynamically from today's Rangers schedule fixture
echo "  Extracting today's Rangers gamePk..."
GAME_PK_PREGAME=$(python3 -c "
import json
with open('$FIXTURE_DIR/schedule_game_today.json') as f:
    data = json.load(f)
dates = data.get('dates', [])
if dates and dates[0].get('games'):
    print(dates[0]['games'][0].get('gamePk', ''))
else:
    print('')
" 2>/dev/null || echo "")

if [ -z "$GAME_PK_PREGAME" ]; then
    echo "  No Rangers game found today — skipping live_game_feed_pregame.json"
    echo "  To create manually:"
    echo "    curl '$BASE_V11/game/<gamePk>/feed/live' | python3 -m json.tool --indent 2 > $FIXTURE_DIR/live_game_feed_pregame.json"
else
    echo "  gamePk = $GAME_PK_PREGAME"
    fetch \
        "$BASE_V11/game/$GAME_PK_PREGAME/feed/live" \
        "live_game_feed_pregame.json"
fi

# ---------------------------------------------------------------------------
# Standalone game endpoints
# ---------------------------------------------------------------------------
echo ""
echo "==> standalone game endpoints"

fetch "$BASE/game/$GAME_PK_FINAL/linescore"      "linescore.json"
fetch "$BASE/game/$GAME_PK_FINAL/boxscore"       "boxscore.json"
fetch "$BASE/game/$GAME_PK_FINAL/playByPlay"     "play_by_play.json"
fetch "$BASE/game/$GAME_PK_FINAL/winProbability" "win_probability.json"
fetch "$BASE/game/$GAME_PK_FINAL/content"        "game_content.json"
fetch "$BASE/game/$GAME_PK_FINAL/contextMetrics" "context_metrics.json"
fetch "$BASE/game/$GAME_PK_FINAL/officials"      "officials.json"

# ---------------------------------------------------------------------------
# Standings
# ---------------------------------------------------------------------------
echo ""
echo "==> standings"

# Both leagues in a single call (comma-separated leagueId)
fetch \
    "$BASE/standings?leagueId=$AL_LEAGUE_ID,$NL_LEAGUE_ID&season=$SEASON" \
    "standings.json"

# ---------------------------------------------------------------------------
# Teams
# ---------------------------------------------------------------------------
echo ""
echo "==> teams"

fetch "$BASE/teams?sportIds=1&season=$SEASON" "teams.json"

# ---------------------------------------------------------------------------
# Roster
# ---------------------------------------------------------------------------
echo ""
echo "==> roster"

fetch \
    "$BASE/teams/$TEAM_ID/roster?rosterType=active&season=$SEASON" \
    "roster_active.json"

fetch \
    "$BASE/teams/$TEAM_ID/roster?rosterType=40Man&season=$SEASON" \
    "roster_40man.json"

# ---------------------------------------------------------------------------
# People / players
# ---------------------------------------------------------------------------
echo ""
echo "==> people"

# Player bio without hydration
fetch \
    "$BASE/people/$TROUT_PLAYER_ID" \
    "player_bio.json"

# Batter with season hitting stats
fetch \
    "$BASE/people/$TROUT_PLAYER_ID?hydrate=stats(group=hitting,type=season)" \
    "player_stats_batter.json"

# Pitcher with season pitching stats
fetch \
    "$BASE/people/$EOVALDI_ID?hydrate=stats(group=pitching,type=season)" \
    "player_stats_pitcher.json"

# ---------------------------------------------------------------------------
# Venue
# ---------------------------------------------------------------------------
echo ""
echo "==> venue"

fetch \
    "$BASE/venues/$GLOBE_LIFE_VENUE_ID?hydrate=fieldInfo" \
    "venue.json"

# ---------------------------------------------------------------------------
# League / division / sport
# ---------------------------------------------------------------------------
echo ""
echo "==> league / division / sport"

fetch "$BASE/leagues?sportId=1"   "leagues.json"
fetch "$BASE/divisions?sportId=1" "divisions.json"
fetch "$BASE/sports"              "sports.json"

# ---------------------------------------------------------------------------
# Season
# ---------------------------------------------------------------------------
echo ""
echo "==> season"

fetch "$BASE/seasons/$SEASON?sportId=1"   "season.json"
fetch "$BASE/seasons?sportId=1&all=true"  "seasons_all.json"

# ---------------------------------------------------------------------------
# Stats
# ---------------------------------------------------------------------------
echo ""
echo "==> stats"

fetch \
    "$BASE/stats?stats=season&group=hitting&season=$SEASON&playerPool=All&sportId=1&limit=5" \
    "stats_hitting_season.json"

fetch \
    "$BASE/stats/leaders?leaderCategories=homeRuns&season=$SEASON&sportId=1&limit=10" \
    "stats_hr_leaders.json"

# ---------------------------------------------------------------------------
# Attendance
# ---------------------------------------------------------------------------
echo ""
echo "==> attendance"

fetch \
    "$BASE/attendance?teamId=$TEAM_ID&season=$SEASON" \
    "attendance.json"

# ---------------------------------------------------------------------------
# Meta
# ---------------------------------------------------------------------------
echo ""
echo "==> meta"

fetch "$BASE/meta?type=gameTypes"  "meta_game_types.json"
fetch "$BASE/meta?type=pitchTypes" "meta_pitch_types.json"
fetch "$BASE/meta?type=positions"  "meta_positions.json"

# ---------------------------------------------------------------------------
# .fixtures-meta.json — read by deserialization tests to know which gamePks
# and IDs were used when the fixtures were captured.
# ---------------------------------------------------------------------------
echo ""
echo "==> writing .fixtures-meta.json"

python3 - <<PYEOF
import json, datetime

meta = {
    "game_pk_final": $GAME_PK_FINAL,
    "game_pk_pregame": int("$GAME_PK_PREGAME") if "$GAME_PK_PREGAME" else None,
    "team_id": $TEAM_ID,
    "al_league_id": $AL_LEAGUE_ID,
    "nl_league_id": $NL_LEAGUE_ID,
    "trout_player_id": $TROUT_PLAYER_ID,
    "eovaldi_player_id": $EOVALDI_ID,
    "globe_life_venue_id": $GLOBE_LIFE_VENUE_ID,
    "season": "$SEASON",
    "fetched_at": datetime.datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ"),
}
with open("$FIXTURE_DIR/.fixtures-meta.json", "w") as f:
    json.dump(meta, f, indent=2)
    f.write("\n")
print("  -> $FIXTURE_DIR/.fixtures-meta.json")
PYEOF

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "=== Fixture fetch complete ==="
echo ""
echo "Files written to $FIXTURE_DIR/:"
ls -lh "$FIXTURE_DIR/"
echo ""
echo "IMPORTANT — verify before committing:"
echo "  1. schedule_no_game.json has empty dates[] (true off-day)"
echo "  2. live_game_feed_pregame.json abstractGameState == 'Preview'  (if present)"
echo "  3. live_game_feed_final.json  abstractGameState == 'Final'"
echo ""
echo "Quick verification:"
echo "  python3 -c \"import json; d=json.load(open('$FIXTURE_DIR/live_game_feed_final.json')); print(d['gameData']['status']['abstractGameState'])\""