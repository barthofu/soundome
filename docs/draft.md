track source: 
- soundcloud
- spotify
- etc


metadate provider:
- musicbrainz
- local




- dedupe
- keep best quality


## Actions

- [ ] add musicbrainz_id to track object (or to track_ref, that should then be named track_ref instead, with a unique constraint on composite platform + id, and do the same for album and artists)



Track dedupe detection:
- lv1. same source id -> pas dl 
- lv2. same track name + artists name + release date
- lv3. same musicbrainz resolution

Album dedupe detection:

Artist dedupe detection:

Soundcloud:
- si artiste similaire:
    - si on a un match musicbrainz, on peut voir s'il existe déjà et simplement le compléter
    - 