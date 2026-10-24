# Trims a full /v1/matches/{id}/metadata response to a fixture: two players
# (slots 10 and 4, who carry mvp_rank and ability_stats in the 2026-10-24
# sample), a few samples per list, so the shape is kept and the file stays small.
.match_info.players |= map(select(.player_slot == 10 or .player_slot == 4))
| .match_info.players[] |= (
    .stats |= .[0:3]
    | .items |= .[0:5]
    | .death_details |= .[0:2]
    | .ability_stats |= .[0:3]
    | .pings |= .[0:3]
    | .book_rewards |= .[0:2]
    | .player_tracked_stats |= .[0:3]
    | .stats_type_stat |= .[0:3]
    | .hero_xp_rewards |= .[0:2]
    | .power_up_buffs |= .[0:2]
    | .accolades |= .[0:2]
  )
| .match_info.match_paths.paths |= .[0:2]
| .match_info.match_paths.paths[] |= (
    .x_pos |= .[0:10] | .y_pos |= .[0:10] | .health |= .[0:10]
    | .move_type |= (if type == "array" then .[0:10] else . end)
    | .combat_type |= (if type == "array" then .[0:10] else . end)
  )
| .match_info.damage_matrix.damage_dealers |= .[0:1]
| .match_info.damage_matrix.sample_time_s |= (if type == "array" then .[0:5] else . end)
| .match_info.damage_matrix.damage_dealers[] |= (.damage_sources |= (if type == "array" then .[0:2] else . end))
| .match_info.objectives |= .[0:3]
| .match_info.custom_user_stats |= .[0:3]
| .match_info.match_tracked_stats |= .[0:3]
| .match_info.watched_death_replays |= .[0:3]
