# Todo

## Features
- [ ] (web) Let edit the artists of an album
- [ ] Add "search song" feature to download a single track by name/artist/album
- [ ] Button to fetch all the missing icons of the artists

## Fixes
- [ ] Permit user to cancel pending tasks (currently not working)
- [ ] Cron definition doesn't seems to affect the last and next run time below
- [ ] Dedupe duplicated refs (same type and same value)
- [ ] Soundcloud artist sync seems to duplicate artist some times even if the original artist in the db has the same soundcloud metadata ref (e.g: KTC)
- [x] Soundcloud artist download doesn't seems to gather all the tracks (e.g: KTC on soundcloud has many tracks but only 4 are downloaded)

## Refacto
- [ ] (web) Redo the reactivity of the whole app and the way the data is fetched from the backend (currently, the app is fetching all the data at once and then filtering it on the frontend, which is not efficient). Any edit to the data should be reflected in the UI without having to reload the whole page or anything.
- [ ] (web) Implement true routing everywhere
- [ ] (web) Compact task list view with a "show more" button to expand the task details + pagination for the task list
- [ ] voir pour renommer les bails de "tag" et "tagger" en "metadata"
- [ ] en plus du "transpose" des metadata, faire un "complete" qui vient simplement compléter les metadata manquantes

## CLI

- [ ] Add manual validation commands (list pending, approve, reject) from the terminal.
- [ ] Add M3U8 export command from CLI with path strategy options.
- [ ] Add playlist utilities: diff between playlists and dated snapshot export.
- [ ] Add shell completion generation for bash/zsh/fish.
- [ ] Add resumable downloads for interrupted playlist exports.
- [ ] Add maintenance commands: integrity check between DB and filesystem, and cleanup of temp/cache.
