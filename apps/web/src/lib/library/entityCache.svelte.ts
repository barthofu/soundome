/**
 * Normalized, reactive entity cache — the single source of truth for every
 * track/album/artist/playlist rendered anywhere in the Library UI.
 *
 * Why: with server-side pagination, the same entity (e.g. an artist) can be
 * referenced from many different lists/pages (a track's embedded artist
 * stub, an album's artist list, the artists tab itself, a drill-down view...).
 * Previously each list held its own full copy fetched once, so editing an
 * entity in one place (e.g. renaming an artist) never propagated to any
 * other already-rendered list without a full reload.
 *
 * Fix: every fetch response is merged ("upserted") into these maps by id,
 * and every list/drill-down view resolves its rows through `getTrack`/
 * `getAlbum`/`getArtist`/`getPlaylist` instead of holding its own copy. A
 * single upsert after an edit is therefore immediately visible everywhere.
 */
import type {
  LibraryTrackDto,
  LibraryAlbumDto,
  LibraryArtistDto,
  LibraryPlaylistDto,
} from '../types';

function createEntityCache() {
  const tracks: Map<number, LibraryTrackDto> = $state(new Map());
  const albums: Map<number, LibraryAlbumDto> = $state(new Map());
  const artists: Map<number, LibraryArtistDto> = $state(new Map());
  const playlists: Map<number, LibraryPlaylistDto> = $state(new Map());

  function upsertTrack(t: LibraryTrackDto) { tracks.set(t.id, t); }
  function upsertAlbum(a: LibraryAlbumDto) { albums.set(a.id, a); }
  function upsertArtist(a: LibraryArtistDto) { artists.set(a.id, a); }
  function upsertPlaylist(p: LibraryPlaylistDto) { playlists.set(p.id, p); }

  function upsertTracks(list: LibraryTrackDto[]) { for (const t of list) upsertTrack(t); }
  function upsertAlbums(list: LibraryAlbumDto[]) { for (const a of list) upsertAlbum(a); }
  function upsertArtists(list: LibraryArtistDto[]) { for (const a of list) upsertArtist(a); }
  function upsertPlaylists(list: LibraryPlaylistDto[]) { for (const p of list) upsertPlaylist(p); }

  function removeTrack(id: number) { tracks.delete(id); }
  function removeAlbum(id: number) { albums.delete(id); }
  function removeArtist(id: number) { artists.delete(id); }
  function removePlaylist(id: number) { playlists.delete(id); }

  function getTrack(id: number): LibraryTrackDto | undefined { return tracks.get(id); }
  function getAlbum(id: number): LibraryAlbumDto | undefined { return albums.get(id); }
  function getArtist(id: number): LibraryArtistDto | undefined { return artists.get(id); }
  function getPlaylist(id: number): LibraryPlaylistDto | undefined { return playlists.get(id); }

  /**
   * Resolve the live display name for an embedded artist stub (`{id, name}`),
   * falling back to the stub's own snapshot when that artist hasn't been
   * individually hydrated into the cache yet.
   */
  function artistName(stub: { id: number | null; name: string }): string {
    if (stub.id == null) return stub.name;
    return artists.get(stub.id)?.name ?? stub.name;
  }

  /** Resolve the live display title for an embedded album stub (`{id, title}`). */
  function albumTitle(stub: { id: number | null; title: string } | null): string | null {
    if (!stub) return null;
    if (stub.id == null) return stub.title;
    return albums.get(stub.id)?.title ?? stub.title;
  }

  /** Resolve the live cover for an embedded album stub, if that album is cached. */
  function albumCover(stub: { id: number | null } | null): string | null | undefined {
    if (!stub || stub.id == null) return undefined;
    return albums.get(stub.id)?.cover;
  }

  return {
    // Raw maps — exposed read-only via getters, for the rare case a
    // component needs to iterate every currently-known entity (e.g. the
    // similarity/duplicate scan works off the dedicated lightweight
    // names endpoints instead, not these maps).
    get tracks() { return tracks; },
    get albums() { return albums; },
    get artists() { return artists; },
    get playlists() { return playlists; },

    upsertTrack, upsertAlbum, upsertArtist, upsertPlaylist,
    upsertTracks, upsertAlbums, upsertArtists, upsertPlaylists,
    removeTrack, removeAlbum, removeArtist, removePlaylist,
    getTrack, getAlbum, getArtist, getPlaylist,
    artistName, albumTitle, albumCover,
  };
}

export const entityCache = createEntityCache();
