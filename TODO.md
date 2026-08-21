# Todo

## Features
- [ ] (web) Let edit the artists of an album
- [ ] Add "search song" feature to download a single track by name/artist/album
- [ ] Button to fetch all the missing icons of the artists

## Fixes
- [ ] Permit user to cancel pending tasks (currently not working)
- [ ] (web) CRON modification doesn't seems to affect the last and next run time below in the sync settings page
- [ ] Dedupe duplicated refs (same type and same value)
- [x] Soundcloud artist sync seems to duplicate artist some times even if the original artist in the db has the same soundcloud metadata ref (e.g: KTC)
- [x] Soundcloud artist download doesn't seems to gather all the tracks (e.g: KTC on soundcloud has many tracks but only 4 are downloaded)

## Refacto
- [ ] (web) Redo the reactivity of the whole app and the way the data is fetched from the backend (currently, the app is fetching all the data at once and then filtering it on the frontend, which is not efficient). Any edit to the data should be reflected in the UI without having to reload the whole page or anything.
- [ ] (web) Implement true routing everywhere
- [ ] (web) Compact task list view with a "show more" button to expand the task details + pagination for the task list
