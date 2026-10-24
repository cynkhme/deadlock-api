# Replaces the personal fields of a list of Steam profiles (names, profile
# and avatar URLs, country, friend ids) with stand-ins of the same type, so
# the published fixtures carry the response's shape and no one's profile.
# Stand-in URLs use the reserved example.invalid host and stand-in ids start
# at 4000000000, above the account ids Steam has assigned, so none resolves
# to a real person. A search response holds strangers, so its account ids
# are replaced too; a profile response keeps them, as do dates and counters.
(input_filename // "" | test("search")) as $search
| (if $search then "search-hit" else "player" end) as $name
| (if $search then 4100000000 else 4000000000 end) as $base
| def stand_in(v): if . == null then null else v end;
def avatar(n; suffix): "https://example.invalid/avatars/\($name)-\(n)\(suffix).jpg";
to_entries
| map(
    (.key + 1) as $n
    | .value
    | if $search then .account_id = $base + $n else . end
    | .personaname |= stand_in("\($name)-\($n)")
    | .realname |= stand_in("Player \($n)")
    | .profileurl |= stand_in("https://example.invalid/id/\($name)-\($n)/")
    | .avatar |= stand_in(avatar($n; ""))
    | .avatarmedium |= stand_in(avatar($n; "_medium"))
    | .avatarfull |= stand_in(avatar($n; "_full"))
    | .countrycode |= stand_in("ZZ")
    | .friends |= (if type == "array"
        then to_entries | map(.value.account_id = $base + $n * 1000 + .key | .value)
        else . end)
  )
