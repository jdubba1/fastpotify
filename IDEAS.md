# Ideas

## Following feed

Show a chronological feed of releases from followed artists.

- Persist each artist's last checked time and known release IDs.
- Refresh a small rotating batch of artists in the background.
- Serve cached results on Home instead of waiting on Spotify.
- Deduplicate singles and album editions, then sort by release date.

Do not fan out across every followed artist on Home load. Large libraries quickly hit Spotify's shared-client rate limit.
