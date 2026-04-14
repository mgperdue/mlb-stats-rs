#!/usr/bin/env bash
# fetch_fixtures.sh
#
# Fetches real MLB Stats API responses and saves them as fixture files for
# use in deserialization unit tests. Run this from the workspace root:
#
#   bash scripts/fetch_fixtures.sh
#
# Fixtures are saved verbatim — no modifications, no formatting. They represent
# exactly what the API returned at the time of the fetch.
#
# Requirements: curl, python3 (for pretty-printing JSON)
#
# The gamePks used here are:
#   823966  — Texas Rangers 5, Los Angeles Dodgers 2 (April 12, 2026, Final)
#   <today> — Rangers at Athletics (April 13, 2026, Pregame/Scheduled)
#
# If you need to refresh fixtures for a different date or game, update the
# GAME_PK_FINAL, TEAM_ID, and DATE_TODAY variables below.

set -euo pipefail

BASE="https://statsapi.mlb.com/api/v1"
BASE_V11="https://statsapi.mlb.com/api/v1.1"
FIXTURE_DIR="mlb-stats-api/tests/fixtures"

GAME_PK_FINAL=823966      # Rangers 5, Dodgers 2 — April 12 2026 (Final)
TEAM_ID=140               # Texas Rangers
DATE_TODAY="04/13/2026"   # MM/DD/YYYY as required by the schedule endpoint
DATE_FINAL="04/12/2026"
SEASON="2026"
LEAGUE_IDS="103,104"      # AL and NL

mkdir -p "$FIXTURE_DIR"

fetch() {
    local url="$1"
    local outfile="$2"
    echo "Fetching: $url"
    curl -s --fail \
        -H "Accept: application/json" \
        "$url" \
        | python3 -m json.tool --indent 2 \
        > "$FIXTURE_DIR/$outfile"
    echo "  -> $FIXTURE_DIR/$outfile"
}

echo "=== Fetching MLB Stats API fixtures ==="
echo ""

# ---------------------------------------------------------------------------
# Schedule fixtures
# ---------------------------------------------------------------------------

# All games today (all 30 teams)
fetch \
    "$BASE/schedule?sportId=1&date=$DATE_TODAY" \
    "schedule_all_games.json"

# Rangers game today (pregame — good for testing pre-game schedule shape)
fetch \
    "$BASE/schedule?sportId=1&date=$DATE_TODAY&teamId=$TEAM_ID" \
    "schedule_game_today.json"

# Rangers game on the final-game date (for correlation with live feed fixture)
fetch \
    "$BASE/schedule?sportId=1&date=$DATE_FINAL&teamId=$TEAM_ID" \
    "schedule_game_final_date.json"

# Off-day: March 27 2026 is a confirmed Rangers off-day — the scheduled
# rest day between the March 26 opener in Philadelphia and the next games.
# The API returns a valid ScheduleResponse with an empty dates[] array.
fetch \
    "$BASE/schedule?sportId=1&date=03/27/2026&teamId=$TEAM_ID" \
    "schedule_no_game.json"

# Schedule with weather + decisions hydration (demonstrates hydrated shape)
fetch \
    "$BASE/schedule?sportId=1&date=$DATE_FINAL&teamId=$TEAM_ID&hydrate=linescore,decisions,weather,broadcasts,officials" \
    "schedule_game_hydrated.json"

# ---------------------------------------------------------------------------
# Live feed fixtures
# ---------------------------------------------------------------------------

# Final game: Rangers 5, Dodgers 2 — complete game, all fields populated
fetch \
    "$BASE_V11/game/$GAME_PK_FINAL/feed/live" \
    "live_game_feed_final.json"

# Get the pregame gamePk from today's Rangers schedule response
echo ""
echo "Extracting today's Rangers gamePk from schedule_game_today.json..."
GAME_PK_PREGAME=$(python3 -c "
import json, sys
with open('$FIXTURE_DIR/schedule_game_today.json') as f:
    data = json.load(f)
dates = data.get('dates', [])
if dates and dates[0].get('games'):
    pk = dates[0]['games'][0].get('gamePk')
    print(pk)
else:
    print('NO_GAME')
" 2>/dev/null || echo "NO_GAME")

if [ "$GAME_PK_PREGAME" = "NO_GAME" ] || [ -z "$GAME_PK_PREGAME" ]; then
    echo "  No Rangers game found today — skipping pregame live feed fixture"
    echo "  To create this fixture manually, run:"
    echo "    curl -s '$BASE_V11/game/<gamePk>/feed/live' | python3 -m json.tool --indent 2 > $FIXTURE_DIR/live_game_feed_pregame.json"
else
    echo "  gamePk: $GAME_PK_PREGAME"
    fetch \
        "$BASE_V11/game/$GAME_PK_PREGAME/feed/live" \
        "live_game_feed_pregame.json"
fi

# ---------------------------------------------------------------------------
# Standalone linescore and boxscore
# ---------------------------------------------------------------------------

fetch \
    "$BASE/game/$GAME_PK_FINAL/linescore" \
    "linescore.json"

fetch \
    "$BASE/game/$GAME_PK_FINAL/boxscore" \
    "boxscore.json"

# ---------------------------------------------------------------------------
# Standings
# ---------------------------------------------------------------------------

fetch \
    "$BASE/standings?leagueId=$LEAGUE_IDS&season=$SEASON" \
    "standings.json"

# ---------------------------------------------------------------------------
# Teams
# ---------------------------------------------------------------------------

fetch \
    "$BASE/teams?sportIds=1&season=$SEASON" \
    "teams.json"

# ---------------------------------------------------------------------------
# Roster
# ---------------------------------------------------------------------------

fetch \
    "$BASE/teams/$TEAM_ID/roster?rosterType=active&season=$SEASON" \
    "roster_active.json"

fetch \
    "$BASE/teams/$TEAM_ID/roster?rosterType=40Man&season=$SEASON" \
    "roster_40man.json"

# ---------------------------------------------------------------------------
# Player / people
# ---------------------------------------------------------------------------

# Nathan Eovaldi (Rangers SP, playerId=543243)
fetch \
    "$BASE/people/543243" \
    "player_bio.json"

# Nathan Eovaldi with pitching stats
fetch \
    "$BASE/people/543243?hydrate=stats(group=pitching,type=season)" \
    "player_stats_pitcher.json"

# Brandon Nimmo (Rangers OF, look up ID from roster if needed)
# Using playerId obtained from the Rangers roster (update if needed)
# Nimmo's MLB ID: 607043
fetch \
    "$BASE/people/607043?hydrate=stats(group=hitting,type=season)" \
    "player_stats_batter.json"

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------

echo ""
echo "=== Fixture fetch complete ==="
echo ""
echo "Files written to $FIXTURE_DIR/:"
ls -lh "$FIXTURE_DIR/"
echo ""
echo "IMPORTANT: Verify the following before committing fixtures:"
echo "  1. schedule_no_game.json has an empty dates[] array (true off-day)"
echo "     If not, pick a different date and re-run with DATE_NO_GAME adjusted."
echo "  2. live_game_feed_pregame.json status.abstractGameState == 'Preview'"
echo "  3. live_game_feed_final.json status.abstractGameState == 'Final'"
echo ""
echo "Suggested verification:"
echo "  python3 -c \\"
echo "    \"import json; d=json.load(open('$FIXTURE_DIR/live_game_feed_final.json')); \\"
echo "     print(d['gameData']['status']['abstractGameState'])\""