#!/bin/sh
# Trims full Assets and patches responses into tests/fixtures/. $RAW holds the
# responses of /v1/assets/heroes, /v1/assets/items, /v1/assets/ranks,
# /v1/patches, /v2/patches and /v1/patches/big-days as heroes.json, items.json,
# ranks.json, patches_v1.json, patches_v2.json and big_days.json. No request is
# made here.
set -eu
RAW=${RAW:?set RAW to the directory of the raw responses}
OUT=${OUT:-tests/fixtures}

# Heroes: a selectable one with the most keys, a selectable one without
# `gender`, and one that cannot be selected.
jq '[ (map(select(.player_selectable)) | max_by(keys | length)),
      (map(select(.player_selectable and (has("gender") | not))) | first),
      (map(select(.player_selectable | not)) | first) ]' "$RAW/heroes.json" > "$OUT/assets_heroes.json"

# Items: per type, the fewest items (picked greedily) that together carry
# every key any item of that type has, so the drift test sees rare fields.
jq 'def cover:
      . as $all
      | ([ $all[] | keys[] ] | unique) as $want
      | { picked: [], have: [] }
      | until(($want - .have) == [];
          ($want - .have) as $missing
          | ($all | max_by([ keys[] as $k | select($missing | index($k)) ] | length)) as $best
          | .picked += [$best]
          | .have = (.have + ($best | keys) | unique))
      | .picked;
    [ "ability", "upgrade", "weapon" ] as $kinds
    | [ $kinds[] as $k | map(select(.type == $k)) | cover[] ]' "$RAW/items.json" > "$OUT/assets_items.json"

# Ranks: the unranked tier 0 and tier 1.
jq 'map(select(.tier <= 1))' "$RAW/ranks.json" > "$OUT/assets_ranks.json"

# Patches: two posts, HTML cut to 200 characters; the v2 feed keeps one forum
# post and one Steam post.
jq '.[0:2] | map(.content_encoded |= .[0:200])' "$RAW/patches_v1.json" > "$OUT/patches.json"
jq '[ (map(select(.source == "forum")) | first), (map(select(.source == "steam")) | first) ]
    | map(.content |= .[0:200])' "$RAW/patches_v2.json" > "$OUT/patch_feed.json"

cp "$RAW/big_days.json" "$OUT/patch_big_days.json"
