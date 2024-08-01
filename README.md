# Shuffles my personal playlist

### Installation for linux cron (daily reshuffles the playlist)
```cron
0 2 * * * sudo -u linus bash -c "cd /home/linus/spotify_shuffler && spotify_shuffler" > /tmp/spotify_shuffler.log 2>&1
```
Clone repo to `/home/linus/spotify_shuffler`.

Build and add `./target/releases/spotify_shuffler` to `/usr/bin` or `/bin`.

Spotify credentials cache file (`.spotify_token_cache.json`) will be created in CWD so make sure you cd into the correct file.
Manual run it once to setup credentials afterwards it will update the key automatically.

