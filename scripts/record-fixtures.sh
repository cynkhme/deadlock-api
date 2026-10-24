#!/bin/sh
# Re-records tests/fixtures/*.json from the live server. Run by hand from the
# maintainer's machine: 17 requests against an ingested match and a known
# account, which the server does not rate-limit the way it limits metadata of
# matches it has not ingested yet. Raw responses are kept in $RAW for review.
set -eu
BASE=${DEADLOCK_API_BASE:-https://api.deadlock-api.com}
MATCH=${MATCH:-122539002}
ACCOUNT=${ACCOUNT:-167576350}
ACCOUNT2=${ACCOUNT2:-3226752420}
SEARCH=${SEARCH:-recrent}
REGION=${REGION:-Europe}   # leaderboard regions are capitalised: Europe, Asia, NAmerica, SAmerica, Oceania
HERO=${HERO:-2}
OUT=${OUT:-tests/fixtures}
RAW=${RAW:-$(mktemp -d)}
UA="deadlock-api-rs/record-fixtures"
mkdir -p "$OUT" "$RAW"

# get <name> <path-with-query>: fetch into $RAW/<name>.json, print the status.
get() {
  code=$(curl -sS -o "$RAW/$1.json" -w '%{http_code}' -H 'Accept: application/json' -A "$UA" "$BASE$2")
  printf '%s\t%s\t%s\n' "$code" "$1" "$2"
  sleep 1
}
# save <name> <jq-filter>: trim $RAW/<name>.json into $OUT/<name>.json.
save() { jq "$2" "$RAW/$1.json" > "$OUT/$1.json"; }

get match_salts            "/v1/matches/$MATCH/salts"
get spectate_url_400       "/v1/matches/$MATCH/live/url"
get active_matches         "/v1/matches/active"
get recently_fetched       "/v1/matches/recently-fetched"
get match_history          "/v1/players/$ACCOUNT/match-history"
get player_rank            "/v1/players/$ACCOUNT/rank"
get player_ranks           "/v1/players/rank?account_ids=$ACCOUNT,$ACCOUNT2"
get rank_distribution      "/v1/players/rank/distribution"
get hero_stats             "/v1/players/hero-stats?account_ids=$ACCOUNT"
get mate_stats             "/v1/players/$ACCOUNT/mate-stats"
get enemy_stats            "/v1/players/$ACCOUNT/enemy-stats"
get steam_profiles         "/v1/players/steam?account_ids=$ACCOUNT,$ACCOUNT2"
get steam_search           "/v1/players/steam-search?search_query=$(printf '%s' "$SEARCH" | jq -sRr @uri)&limit=3"
get leaderboard            "/v1/leaderboard/$REGION"
get hero_leaderboard       "/v1/leaderboard/$REGION/$HERO"
get info                   "/v1/info"
get health                 "/v1/info/health"

save match_salts        '.'
save spectate_url_400   '.'
save active_matches     '.[0:3]'
save recently_fetched   '.[0:3]'
save match_history      '.[0:3]'
save player_rank        '.'
save player_ranks       '.'
save rank_distribution  '.[0:5]'
save hero_stats         '.[0:3]'
save mate_stats         '.[0:3] | map(.matches |= (if type=="array" then .[0:5] else . end))'
save enemy_stats        '.[0:3] | map(.matches |= (if type=="array" then .[0:5] else . end))'
save steam_profiles     'map(.friends |= (if type=="array" then .[0:2] else . end))'
save steam_search       '.[0:1] | map(.friends |= (if type=="array" then .[0:2] else . end))'
# The Steam fixtures are published with the crate: replace the personal fields.
for name in steam_profiles steam_search; do
  jq -f "$(dirname "$0")/pseudonymise-steam.jq" "$OUT/$name.json" > "$RAW/$name.pseudonymised.json"
  mv "$RAW/$name.pseudonymised.json" "$OUT/$name.json"
done
# Leaderboard names are players' names too: each becomes player-<rank>.
save leaderboard        '.entries |= (if type=="array" then .[0:3] | map(.rank as $r | .account_name |= (if . == null then null else "player-\($r)" end)) else . end)'
save hero_leaderboard   '.entries |= (if type=="array" then .[0:3] | map(.rank as $r | .account_name |= (if . == null then null else "player-\($r)" end)) else . end)'
save info               '.'
save health             '.'
echo "raw responses: $RAW"
