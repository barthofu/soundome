import {
  getTracksPage, updateTrack, deleteTrack,
  getAlbumsPage, updateAlbum, deleteAlbum, mergeAlbums, getAlbumNames, getAlbumTracks,
  getArtistsPage, updateArtist, deleteArtist, mergeArtists, getArtistNames, getArtistTracks, getArtistAlbums,
  uploadArtistImage, uploadAlbumImage, uploadTrackImage,
  fetchArtistIconFromReferences, fetchAlbumCoverFromReferences,
  batchFetchArtistIcons, batchFetchAlbumCovers,
  getPlaylistsPage, getPlaylistTracks, deletePlaylist,
  addEntityReference, deleteEntityReference,
  getPendingCount,
} from '../api';
import type {
  LibraryTrackDto, UpdateTrackBody,
  LibraryAlbumDto, UpdateAlbumBody,
  LibraryArtistDto, UpdateArtistBody,
  LibraryPlaylistDto, PlaylistTrackDto,
  ReferenceDto, AddReferenceBody,
} from '../types';
import { entityCache } from './entityCache.svelte';
import { debounce } from '../utils/debounce';

export type Tab = 'artists' | 'albums' | 'tracks' | 'playlists';
export type ViewMode = 'list' | 'grid';
export type TrackFilter = 'all' | 'ok' | 'pending';
export type SortDirection = 'asc' | 'desc';
export type ArtistSortBy = 'name' | 'track_count' | 'album_count';
export type AlbumSortBy = 'title' | 'date' | 'artist' | 'track_count';
export type TrackSortBy = 'title' | 'artist' | 'album' | 'date' | 'duration';
export type EditState =
  | { type: 'track'; item: LibraryTrackDto }
  | { type: 'album'; item: LibraryAlbumDto }
  | { type: 'artist'; item: LibraryArtistDto }
  | null;
export type HoveredItem = { type: 'track' | 'album' | 'artist'; id: number } | null;

const PAGE_SIZE = 60;

// ── Artist/album name similarity helpers ──────────────────────────────────────
function _editDistance(a: string, b: string): number {
  const m = a.length, n = b.length;
  const dp: number[] = Array.from({ length: n + 1 }, (_, i) => i);
  for (let i = 1; i <= m; i++) {
    let prev = dp[0]; dp[0] = i;
    for (let j = 1; j <= n; j++) {
      const tmp = dp[j];
      dp[j] = a[i - 1] === b[j - 1] ? prev : 1 + Math.min(prev, dp[j], dp[j - 1]);
      prev = tmp;
    }
  }
  return dp[n];
}

export function areSimilarArtistNames(a: string, b: string): boolean {
  const norm = (s: string) => s.toLowerCase().replace(/[^a-z0-9]/g, '');
  const na = norm(a), nb = norm(b);
  if (na === nb) return true;
  if (na.length < 2 || nb.length < 2) return false;
  const dist = _editDistance(na, nb);
  const maxLen = Math.max(na.length, nb.length);
  return dist <= 2 || (maxLen >= 8 && dist / maxLen <= 0.2);
}

// Album title similarity mirrors artist name similarity.
export function areSimilarAlbumNames(a: string, b: string): boolean {
  return areSimilarArtistNames(a, b);
}

function createLibraryStore() {
  const _initHash = (() => {
    const raw = location.hash.replace('#', '');
    const p = raw.split('/');
    const t = (['artists', 'albums', 'tracks', 'playlists'] as const).find(x => x === p[0]) ?? 'albums';
    const aid = t === 'artists' && p[1] ? (parseInt(p[1]) || null) : null;
    const bid =
      t === 'albums' && p[1] ? (parseInt(p[1]) || null) :
      t === 'artists' && p[2] === 'album' && p[3] ? (parseInt(p[3]) || null) : null;
    const pid = t === 'playlists' && p[1] ? (parseInt(p[1]) || null) : null;
    return { tab: t, artistId: aid, albumId: bid, playlistId: pid };
  })();

  // ── State ──────────────────────────────────────────────────────────────────
  let tab: Tab = $state(_initHash.tab);
  let tracksView: ViewMode = $state('list');
  let albumsView: ViewMode = $state('grid');
  let artistsView: ViewMode = $state('grid');

  // Sort/filter/search state
  let artistsSortBy: ArtistSortBy = $state('name');
  let artistsSortDir: SortDirection = $state('asc');
  let albumsSortBy: AlbumSortBy = $state('title');
  let albumsSortDir: SortDirection = $state('asc');
  let tracksSortBy: TrackSortBy = $state('title');
  let tracksSortDir: SortDirection = $state('asc');
  let trackSearch = $state('');
  let albumSearch = $state('');
  let artistSearch = $state('');
  let playlistSearch = $state('');
  let trackFilter: TrackFilter = $state('ok');

  // ── Paginated list state (ids into the normalized entityCache) ─────────────
  let trackIds: number[] = $state([]);
  let tracksTotal = $state(0);
  let tracksPage = $state(0);
  let tracksLoaded = $state(false);
  let tracksLoading = $state(false);
  let tracksLoadingMore = $state(false);
  let tracksError: string | null = $state(null);

  let albumIds: number[] = $state([]);
  let albumsTotal = $state(0);
  let albumsPage = $state(0);
  let albumsLoaded = $state(false);
  let albumsLoading = $state(false);
  let albumsLoadingMore = $state(false);
  let albumsError: string | null = $state(null);

  let artistIds: number[] = $state([]);
  let artistsTotal = $state(0);
  let artistsPage = $state(0);
  let artistsLoaded = $state(false);
  let artistsLoading = $state(false);
  let artistsLoadingMore = $state(false);
  let artistsError: string | null = $state(null);

  let playlistIds: number[] = $state([]);
  let playlistsTotal = $state(0);
  let playlistsPage = $state(0);
  let playlistsLoaded = $state(false);
  let playlistsLoading = $state(false);
  let playlistsLoadingMore = $state(false);
  let playlistsError: string | null = $state(null);

  let pendingCount = $state(0);

  let drillPlaylistId: number | null = $state(_initHash.playlistId);
  let drillPlaylistTracks: PlaylistTrackDto[] = $state([]);
  let drillPlaylistTracksLoading = $state(false);
  let drillPlaylistTracksError: string | null = $state(null);

  let drillArtistId: number | null = $state(_initHash.artistId);
  let drillAlbumId: number | null = $state(_initHash.albumId);
  let drillArtistAlbumIds: number[] = $state([]);
  let drillArtistTrackIds: number[] = $state([]);
  let drillAlbumTrackIds: number[] = $state([]);
  let drillDataLoading = $state(false);

  // ── Global refresh state ───────────────────────────────────────────────────
  let refreshing = $state(false);
  let lastRefreshed: Date | null = $state(null);

  let editState: EditState = $state(null);
  let editSaving = $state(false);
  let imageUploading = $state(false);
  let thumbnailFetching = $state(false);
  let trackDraft: UpdateTrackBody = $state({});
  let albumDraft: UpdateAlbumBody = $state({});
  let artistDraft: UpdateArtistBody = $state({});

  let batchFetchingArtists = $state(false);
  let batchFetchingAlbums = $state(false);
  let batchFetchResult: { count: number; skipped: number } | null = $state(null);

  let hoveredItem: HoveredItem = $state(null);

  // ── Artist selection / merge state ─────────────────────────────────────────
  let selectedArtistIds: Set<number> = $state(new Set());
  let mergePicking = $state(false);
  let mergeSaving = $state(false);
  let similarFilterActive = $state(false);

  // ── Album selection / merge state ──────────────────────────────────────────
  let selectedAlbumIds: Set<number> = $state(new Set());
  let albumMergePicking = $state(false);
  let albumMergeSaving = $state(false);
  let albumSimilarFilterActive = $state(false);

  // ── Lightweight id+name lists (duplicate detection + artist autocomplete) ──
  let artistNames: { id: number; name: string }[] = $state([]);
  let artistNamesLoaded = $state(false);
  let albumNames: { id: number; title: string }[] = $state([]);
  let albumNamesLoaded = $state(false);

  async function ensureArtistNames() {
    if (artistNamesLoaded) return;
    try { artistNames = await getArtistNames(); artistNamesLoaded = true; } catch { /* best-effort */ }
  }
  async function ensureAlbumNames() {
    if (albumNamesLoaded) return;
    try { albumNames = await getAlbumNames(); albumNamesLoaded = true; } catch { /* best-effort */ }
  }
  /** Called after an artist create/rename/merge so autocomplete stays fresh. */
  function invalidateArtistNames() { artistNamesLoaded = false; }
  function invalidateAlbumNames() { albumNamesLoaded = false; }

  // ── Derived: resolve paginated ids through the normalized cache ────────────
  let filteredTracks = $derived(
    trackIds.map(id => entityCache.getTrack(id)).filter((t): t is LibraryTrackDto => t != null)
  );
  let filteredAlbums = $derived(
    albumIds.map(id => entityCache.getAlbum(id)).filter((a): a is LibraryAlbumDto => a != null)
  );
  let filteredArtists = $derived(
    artistIds.map(id => entityCache.getArtist(id)).filter((a): a is LibraryArtistDto => a != null)
  );
  let filteredPlaylists = $derived(
    playlistIds.map(id => entityCache.getPlaylist(id)).filter((p): p is LibraryPlaylistDto => p != null)
  );

  let tracksHasMore = $derived(trackIds.length < tracksTotal);
  let albumsHasMore = $derived(albumIds.length < albumsTotal);
  let artistsHasMore = $derived(artistIds.length < artistsTotal);
  let playlistsHasMore = $derived(playlistIds.length < playlistsTotal);

  let drillArtist = $derived(drillArtistId != null ? entityCache.getArtist(drillArtistId) ?? null : null);
  let drillAlbum = $derived(drillAlbumId != null ? entityCache.getAlbum(drillAlbumId) ?? null : null);
  let drillPlaylist = $derived(drillPlaylistId != null ? entityCache.getPlaylist(drillPlaylistId) ?? null : null);

  let artistAlbums = $derived(
    drillArtistAlbumIds.map(id => entityCache.getAlbum(id)).filter((a): a is LibraryAlbumDto => a != null)
  );
  let artistTracks = $derived(
    drillArtistTrackIds.map(id => entityCache.getTrack(id)).filter((t): t is LibraryTrackDto => t != null)
  );
  let albumTracks = $derived(
    drillAlbumTrackIds.map(id => entityCache.getTrack(id)).filter((t): t is LibraryTrackDto => t != null)
  );
  let artistTracksByAlbum = $derived.by(() => {
    type Group = { albumId: number | null; albumTitle: string | null; albumCover: string | null; tracks: LibraryTrackDto[] };
    const map = new Map<string, Group>();
    for (const t of artistTracks) {
      const key = t.album?.id != null ? String(t.album.id) : '__none__';
      if (!map.has(key)) {
        const full = t.album?.id != null ? entityCache.getAlbum(t.album.id) : null;
        map.set(key, {
          albumId: t.album?.id ?? null,
          albumTitle: entityCache.albumTitle(t.album) ?? t.album?.title ?? null,
          albumCover: full?.cover ?? null,
          tracks: [],
        });
      }
      map.get(key)!.tracks.push(t);
    }
    const result = [...map.values()];
    result.sort((a, b) => {
      if (a.albumId === null) return 1;
      if (b.albumId === null) return -1;
      const aDate = (a.albumId != null ? entityCache.getAlbum(a.albumId)?.date : null) ?? '';
      const bDate = (b.albumId != null ? entityCache.getAlbum(b.albumId)?.date : null) ?? '';
      if (aDate !== bDate) return aDate < bDate ? -1 : 1;
      return (a.albumTitle ?? '') < (b.albumTitle ?? '') ? -1 : 1;
    });
    return result;
  });

  let similarArtistIds = $derived.by(() => {
    const ids = new Set<number>();
    for (let i = 0; i < artistNames.length; i++) {
      for (let j = i + 1; j < artistNames.length; j++) {
        if (areSimilarArtistNames(artistNames[i].name, artistNames[j].name)) {
          ids.add(artistNames[i].id);
          ids.add(artistNames[j].id);
        }
      }
    }
    return ids;
  });
  let similarAlbumIds = $derived.by(() => {
    const ids = new Set<number>();
    for (let i = 0; i < albumNames.length; i++) {
      for (let j = i + 1; j < albumNames.length; j++) {
        if (areSimilarAlbumNames(albumNames[i].title, albumNames[j].title)) {
          ids.add(albumNames[i].id);
          ids.add(albumNames[j].id);
        }
      }
    }
    return ids;
  });

  // ── URL navigation ─────────────────────────────────────────────────────────
  function buildHash(t: Tab, artistId?: number, albumId?: number, playlistId?: number): string {
    if (t === 'artists') {
      if (artistId && albumId) return `#artists/${artistId}/album/${albumId}`;
      if (artistId) return `#artists/${artistId}`;
      return '#artists';
    }
    if (t === 'albums') { if (albumId) return `#albums/${albumId}`; return '#albums'; }
    if (t === 'playlists') { if (playlistId) return `#playlists/${playlistId}`; return '#playlists'; }
    return '#tracks';
  }
  function navigate(t: Tab, artistId?: number, albumId?: number, playlistId?: number) {
    const h = buildHash(t, artistId, albumId, playlistId);
    if (location.hash !== h) history.pushState(null, '', h);
    tab = t; drillArtistId = artistId ?? null; drillAlbumId = albumId ?? null;
    drillPlaylistId = playlistId ?? null; editState = null;
    triggerDrillLoads();
  }
  function applyHash() {
    editState = null;
    const raw = location.hash.replace('#', '');
    if (!raw) { tab = 'albums'; drillArtistId = null; drillAlbumId = null; drillPlaylistId = null; return; }
    const p = raw.split('/');
    const t = (['artists', 'albums', 'tracks', 'playlists'] as const).find(x => x === p[0]) ?? 'albums';
    if (t === 'tracks') { tab = 'tracks'; drillArtistId = null; drillAlbumId = null; drillPlaylistId = null; return; }
    if (t === 'playlists') {
      tab = 'playlists'; drillArtistId = null; drillAlbumId = null;
      drillPlaylistId = p[1] ? (parseInt(p[1]) || null) : null;
      triggerDrillLoads();
      return;
    }
    if (t === 'albums') {
      tab = 'albums'; drillArtistId = null; drillAlbumId = p[1] ? (parseInt(p[1]) || null) : null; drillPlaylistId = null;
      triggerDrillLoads();
      return;
    }
    tab = 'artists';
    drillArtistId = p[1] ? (parseInt(p[1]) || null) : null;
    drillAlbumId = (p[2] === 'album' && p[3]) ? (parseInt(p[3]) || null) : null;
    drillPlaylistId = null;
    triggerDrillLoads();
  }
  /** Fetch whatever drill-down data the current URL/navigation state requires. */
  function triggerDrillLoads() {
    if (drillPlaylistId != null) loadDrillPlaylistTracks(drillPlaylistId);
    if (drillArtistId != null) loadDrillArtist(drillArtistId);
    if (drillAlbumId != null) loadDrillAlbum(drillAlbumId);
  }
  function switchTab(t: Tab) { navigate(t); clearArtistSelection(); clearAlbumSelection(); }
  function clearDrill() { navigate(tab); }

  function handleRefresh() {
    clearDrill();
    loadAll();
  }

  async function loadAll() {
    refreshing = true;
    try {
      await Promise.all([resetTracks(), resetAlbums(), resetArtists(), resetPlaylists(), loadPendingCount()]);
      triggerDrillLoads();
      lastRefreshed = new Date();
    } finally {
      refreshing = false;
    }
  }

  async function loadPendingCount() {
    try { pendingCount = await getPendingCount(); } catch { /* best-effort */ }
  }

  function drillIntoArtist(a: LibraryArtistDto) { navigate('artists', a.id); }
  function drillIntoAlbum(album: LibraryAlbumDto) {
    if (tab === 'albums') navigate('albums', undefined, album.id);
    else navigate('artists', drillArtistId ?? undefined, album.id);
  }
  function backToArtist() { if (drillArtistId) navigate('artists', drillArtistId); }
  function backToRoot() { navigate(tab); }

  // ── Data loading: paginated tracks ──────────────────────────────────────────
  async function resetTracks() {
    tracksLoading = true; tracksError = null; tracksPage = 0; trackIds = [];
    try {
      const result = await getTracksPage({
        page: 1, pageSize: PAGE_SIZE,
        q: trackSearch.trim() || undefined,
        sortBy: tracksSortBy, sortDir: tracksSortDir, filter: trackFilter,
      });
      entityCache.upsertTracks(result.items);
      trackIds = result.items.map(t => t.id);
      tracksTotal = result.total;
      tracksPage = 1;
      tracksLoaded = true;
    } catch (e) {
      tracksError = e instanceof Error ? e.message : String(e);
      tracksLoaded = true;
    } finally {
      tracksLoading = false;
    }
  }
  async function loadMoreTracks() {
    if (tracksLoadingMore || tracksLoading || !tracksHasMore) return;
    tracksLoadingMore = true;
    try {
      const nextPage = tracksPage + 1;
      const result = await getTracksPage({
        page: nextPage, pageSize: PAGE_SIZE,
        q: trackSearch.trim() || undefined,
        sortBy: tracksSortBy, sortDir: tracksSortDir, filter: trackFilter,
      });
      entityCache.upsertTracks(result.items);
      trackIds = [...trackIds, ...result.items.map(t => t.id)];
      tracksTotal = result.total;
      tracksPage = nextPage;
    } catch {
      // Non-fatal: leave the currently-loaded page(s) as-is.
    } finally {
      tracksLoadingMore = false;
    }
  }
  const debouncedResetTracks = debounce(() => resetTracks(), 300);

  // ── Data loading: paginated albums ──────────────────────────────────────────
  async function resetAlbums() {
    albumsLoading = true; albumsError = null; albumsPage = 0; albumIds = [];
    try {
      const result = await getAlbumsPage({
        page: 1, pageSize: PAGE_SIZE,
        q: albumSearch.trim() || undefined,
        sortBy: albumsSortBy, sortDir: albumsSortDir,
      });
      entityCache.upsertAlbums(result.items);
      albumIds = result.items.map(a => a.id);
      albumsTotal = result.total;
      albumsPage = 1;
      albumsLoaded = true;
    } catch (e) {
      albumsError = e instanceof Error ? e.message : String(e);
      albumsLoaded = true;
    } finally {
      albumsLoading = false;
    }
  }
  async function loadMoreAlbums() {
    if (albumsLoadingMore || albumsLoading || !albumsHasMore) return;
    albumsLoadingMore = true;
    try {
      const nextPage = albumsPage + 1;
      const result = await getAlbumsPage({
        page: nextPage, pageSize: PAGE_SIZE,
        q: albumSearch.trim() || undefined,
        sortBy: albumsSortBy, sortDir: albumsSortDir,
      });
      entityCache.upsertAlbums(result.items);
      albumIds = [...albumIds, ...result.items.map(a => a.id)];
      albumsTotal = result.total;
      albumsPage = nextPage;
    } catch {
      // Non-fatal.
    } finally {
      albumsLoadingMore = false;
    }
  }
  const debouncedResetAlbums = debounce(() => resetAlbums(), 300);

  // ── Data loading: paginated artists ─────────────────────────────────────────
  async function resetArtists() {
    artistsLoading = true; artistsError = null; artistsPage = 0; artistIds = [];
    try {
      const result = await getArtistsPage({
        page: 1, pageSize: PAGE_SIZE,
        q: artistSearch.trim() || undefined,
        sortBy: artistsSortBy, sortDir: artistsSortDir,
      });
      entityCache.upsertArtists(result.items);
      artistIds = result.items.map(a => a.id);
      artistsTotal = result.total;
      artistsPage = 1;
      artistsLoaded = true;
    } catch (e) {
      artistsError = e instanceof Error ? e.message : String(e);
      artistsLoaded = true;
    } finally {
      artistsLoading = false;
    }
  }
  async function loadMoreArtists() {
    if (artistsLoadingMore || artistsLoading || !artistsHasMore) return;
    artistsLoadingMore = true;
    try {
      const nextPage = artistsPage + 1;
      const result = await getArtistsPage({
        page: nextPage, pageSize: PAGE_SIZE,
        q: artistSearch.trim() || undefined,
        sortBy: artistsSortBy, sortDir: artistsSortDir,
      });
      entityCache.upsertArtists(result.items);
      artistIds = [...artistIds, ...result.items.map(a => a.id)];
      artistsTotal = result.total;
      artistsPage = nextPage;
    } catch {
      // Non-fatal.
    } finally {
      artistsLoadingMore = false;
    }
  }
  const debouncedResetArtists = debounce(() => resetArtists(), 300);

  // ── Data loading: paginated playlists ───────────────────────────────────────
  async function resetPlaylists() {
    playlistsLoading = true; playlistsError = null; playlistsPage = 0; playlistIds = [];
    try {
      const result = await getPlaylistsPage({
        page: 1, pageSize: PAGE_SIZE,
        q: playlistSearch.trim() || undefined,
      });
      entityCache.upsertPlaylists(result.items);
      playlistIds = result.items.map(p => p.id);
      playlistsTotal = result.total;
      playlistsPage = 1;
      playlistsLoaded = true;
    } catch (e) {
      playlistsError = e instanceof Error ? e.message : String(e);
      playlistsLoaded = true;
    } finally {
      playlistsLoading = false;
    }
  }
  async function loadMorePlaylists() {
    if (playlistsLoadingMore || playlistsLoading || !playlistsHasMore) return;
    playlistsLoadingMore = true;
    try {
      const nextPage = playlistsPage + 1;
      const result = await getPlaylistsPage({
        page: nextPage, pageSize: PAGE_SIZE,
        q: playlistSearch.trim() || undefined,
      });
      entityCache.upsertPlaylists(result.items);
      playlistIds = [...playlistIds, ...result.items.map(p => p.id)];
      playlistsTotal = result.total;
      playlistsPage = nextPage;
    } catch {
      // Non-fatal.
    } finally {
      playlistsLoadingMore = false;
    }
  }
  const debouncedResetPlaylists = debounce(() => resetPlaylists(), 300);

  // ── Drill-down data (artist/album detail views) ─────────────────────────────
  async function loadDrillArtist(id: number) {
    drillDataLoading = true;
    try {
      const [albums, tracks] = await Promise.all([getArtistAlbums(id), getArtistTracks(id)]);
      entityCache.upsertAlbums(albums);
      entityCache.upsertTracks(tracks);
      drillArtistAlbumIds = albums.map(a => a.id);
      drillArtistTrackIds = tracks.map(t => t.id);
    } catch {
      drillArtistAlbumIds = [];
      drillArtistTrackIds = [];
    } finally {
      drillDataLoading = false;
    }
  }
  async function loadDrillAlbum(id: number) {
    drillDataLoading = true;
    try {
      const tracks = await getAlbumTracks(id);
      entityCache.upsertTracks(tracks);
      drillAlbumTrackIds = tracks.map(t => t.id);
    } catch {
      drillAlbumTrackIds = [];
    } finally {
      drillDataLoading = false;
    }
  }
  async function loadDrillPlaylistTracks(id: number) {
    drillPlaylistTracksLoading = true; drillPlaylistTracksError = null;
    try {
      drillPlaylistTracks = await getPlaylistTracks(id);
    } catch (e) {
      drillPlaylistTracksError = e instanceof Error ? e.message : String(e);
    } finally {
      drillPlaylistTracksLoading = false;
    }
  }
  function drillIntoPlaylist(p: LibraryPlaylistDto) {
    drillPlaylistTracks = [];
    navigate('playlists', undefined, undefined, p.id);
    loadDrillPlaylistTracks(p.id);
  }

  // ── Edit helpers ───────────────────────────────────────────────────────────
  function startEditTrack(t: LibraryTrackDto) {
    trackDraft = {
      title: t.title, artists: t.artists.map(a => a.name),
      album_title: t.album?.title ?? undefined, genre: t.genre ?? undefined,
      date: t.date ?? undefined, track_number: t.track_number ?? undefined,
      disc_number: t.disc_number ?? undefined, label: t.label ?? undefined,
      cover: t.cover ?? undefined,
    };
    editState = { type: 'track', item: t };
  }
  function startEditAlbum(a: LibraryAlbumDto) {
    albumDraft = { title: a.title, date: a.date ?? undefined, cover: a.cover ?? undefined };
    editState = { type: 'album', item: a };
  }
  function startEditArtist(a: LibraryArtistDto) {
    artistDraft = { name: a.name, icon: a.icon ?? undefined };
    editState = { type: 'artist', item: a };
  }
  function openEditForHovered() {
    if (!hoveredItem) return;
    if (hoveredItem.type === 'track') { const t = entityCache.getTrack(hoveredItem.id); if (t) startEditTrack(t); }
    else if (hoveredItem.type === 'album') { const a = entityCache.getAlbum(hoveredItem.id); if (a) startEditAlbum(a); }
    else { const a = entityCache.getArtist(hoveredItem.id); if (a) startEditArtist(a); }
  }
  async function saveEdit() {
    if (!editState) return;
    const state = editState;
    editSaving = true;
    try {
      if (state.type === 'track') {
        const updated = await updateTrack(state.item.id, trackDraft);
        entityCache.upsertTrack(updated);
      } else if (state.type === 'album') {
        const updated = await updateAlbum(state.item.id, albumDraft);
        entityCache.upsertAlbum(updated);
        invalidateAlbumNames();
      } else {
        const updated = await updateArtist(state.item.id, artistDraft);
        entityCache.upsertArtist(updated);
        invalidateArtistNames();
      }
      editState = null;
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    } finally { editSaving = false; }
  }

  // ── Image upload ──────────────────────────────────────────────────────────
  async function uploadImage(file: File) {
    if (!editState) return;
    const state = editState;
    imageUploading = true;
    try {
      if (state.type === 'artist') {
        const { url } = await uploadArtistImage(state.item.id, file);
        const updated = { ...state.item, icon: url };
        entityCache.upsertArtist(updated);
        editState = { type: 'artist', item: updated };
        artistDraft.icon = url;
      } else if (state.type === 'album') {
        const { url } = await uploadAlbumImage(state.item.id, file);
        const updated = { ...state.item, cover: url };
        entityCache.upsertAlbum(updated);
        editState = { type: 'album', item: updated };
        albumDraft.cover = url;
      } else {
        const { url } = await uploadTrackImage(state.item.id, file);
        const updated = { ...state.item, cover: url };
        entityCache.upsertTrack(updated);
        editState = { type: 'track', item: updated };
        trackDraft.cover = url;
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      imageUploading = false;
    }
  }

  /**
   * Best-effort: ask the backend to resolve a thumbnail from the currently edited
   * entity's existing references (Spotify, SoundCloud, YouTube Music) and persist it.
   * Only supported for artists (icon) and albums (cover).
   */
  async function fetchThumbnailFromReferences() {
    if (!editState) return;
    const state = editState;
    thumbnailFetching = true;
    try {
      if (state.type === 'artist') {
        const { url } = await fetchArtistIconFromReferences(state.item.id);
        const updated = { ...state.item, icon: url };
        entityCache.upsertArtist(updated);
        editState = { type: 'artist', item: updated };
        artistDraft.icon = url;
      } else if (state.type === 'album') {
        const { url } = await fetchAlbumCoverFromReferences(state.item.id);
        const updated = { ...state.item, cover: url };
        entityCache.upsertAlbum(updated);
        editState = { type: 'album', item: updated };
        albumDraft.cover = url;
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      thumbnailFetching = false;
    }
  }

  /**
   * Batch fetch: for each artist without an icon, try to resolve one from its
   * existing references. Refreshes the artist list after completion.
   */
  async function batchFetchArtistIconsAction() {
    batchFetchingArtists = true;
    batchFetchResult = null;
    try {
      const result = await batchFetchArtistIcons();
      batchFetchResult = result;
      await resetArtists();
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      batchFetchingArtists = false;
    }
  }

  /**
   * Batch fetch: for each album without a cover, try to resolve one from its
   * existing references. Refreshes the album list after completion.
   */
  async function batchFetchAlbumCoversAction() {
    batchFetchingAlbums = true;
    batchFetchResult = null;
    try {
      const result = await batchFetchAlbumCovers();
      batchFetchResult = result;
      await resetAlbums();
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      batchFetchingAlbums = false;
    }
  }

  // ── Delete handlers ────────────────────────────────────────────────────────
  async function handleDeleteTrack(id: number) {
    if (!confirm('Delete this track from the library?')) return;
    try {
      await deleteTrack(id);
      entityCache.removeTrack(id);
      trackIds = trackIds.filter(x => x !== id);
      tracksTotal = Math.max(0, tracksTotal - 1);
      drillArtistTrackIds = drillArtistTrackIds.filter(x => x !== id);
      drillAlbumTrackIds = drillAlbumTrackIds.filter(x => x !== id);
      drillPlaylistTracks = drillPlaylistTracks.filter(t => t.id !== id);
      loadPendingCount();
    } catch (e) { alert(e instanceof Error ? e.message : String(e)); }
  }
  async function handleDeleteAlbum(id: number) {
    if (!confirm('Delete this album? Tracks will remain but lose their album association.')) return;
    try {
      await deleteAlbum(id);
      entityCache.removeAlbum(id);
      albumIds = albumIds.filter(x => x !== id);
      albumsTotal = Math.max(0, albumsTotal - 1);
      drillArtistAlbumIds = drillArtistAlbumIds.filter(x => x !== id);
      invalidateAlbumNames();
      if (drillAlbumId === id) navigate(tab, drillArtistId ?? undefined);
    } catch (e) { alert(e instanceof Error ? e.message : String(e)); }
  }
  async function handleDeleteArtist(id: number) {
    if (!confirm('Delete this artist?')) return;
    try {
      await deleteArtist(id);
      entityCache.removeArtist(id);
      artistIds = artistIds.filter(x => x !== id);
      artistsTotal = Math.max(0, artistsTotal - 1);
      invalidateArtistNames();
      if (drillArtistId === id) navigate('artists');
    } catch (e) { alert(e instanceof Error ? e.message : String(e)); }
  }
  async function handleDeletePlaylist(id: number) {
    if (!confirm('Delete this playlist?')) return;
    const deleteTracks = confirm(
      'Also delete the tracks that belong to this playlist?\n\n' +
      'OK → delete the playlist AND its tracks\n' +
      'Cancel → delete only the playlist (tracks are kept)'
    );
    try {
      let trackIdsToRemove: Set<number> = new Set();
      if (deleteTracks) {
        const source =
          drillPlaylistId === id && drillPlaylistTracks.length > 0
            ? drillPlaylistTracks
            : await getPlaylistTracks(id);
        trackIdsToRemove = new Set(source.map(t => t.id));
      }

      await deletePlaylist(id, deleteTracks);
      entityCache.removePlaylist(id);
      playlistIds = playlistIds.filter(x => x !== id);
      playlistsTotal = Math.max(0, playlistsTotal - 1);

      if (deleteTracks && trackIdsToRemove.size > 0) {
        for (const tid of trackIdsToRemove) entityCache.removeTrack(tid);
        trackIds = trackIds.filter(x => !trackIdsToRemove.has(x));
      }

      if (drillPlaylistId === id) navigate('playlists');

      // Full refresh: playlist deletion can change track counts across tabs.
      loadAll();
    } catch (e) { alert(e instanceof Error ? e.message : String(e)); }
  }

  // ── Artist selection helpers ───────────────────────────────────────────────
  function toggleArtistSelection(id: number) {
    const next = new Set(selectedArtistIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedArtistIds = next;
    if (selectedArtistIds.size < 2) mergePicking = false;
  }
  function clearArtistSelection() {
    selectedArtistIds = new Set();
    mergePicking = false;
  }
  function startMergePicking() {
    if (selectedArtistIds.size >= 2) mergePicking = true;
  }
  function cancelMergePicking() {
    mergePicking = false;
  }
  async function pickMergeTarget(targetId: number) {
    if (!mergePicking || !selectedArtistIds.has(targetId)) return;
    const sourceIds = [...selectedArtistIds].filter(id => id !== targetId);
    const targetName = entityCache.getArtist(targetId)?.name ?? String(targetId);
    const sourceNames = sourceIds.map(id => entityCache.getArtist(id)?.name ?? String(id)).join(', ');
    if (!confirm(`Merge "${sourceNames}" into "${targetName}"?\n\nThis cannot be undone.`)) return;
    mergeSaving = true;
    try {
      const updated = await mergeArtists(sourceIds, targetId);
      for (const sid of sourceIds) entityCache.removeArtist(sid);
      entityCache.upsertArtist(updated);
      artistIds = artistIds.filter(id => !sourceIds.includes(id));
      artistsTotal = Math.max(0, artistsTotal - sourceIds.length);
      invalidateArtistNames();
      clearArtistSelection();
    } catch (e) {
      alert(e instanceof Error ? e.message : String(e));
    } finally {
      mergeSaving = false;
    }
  }

  // ── Album selection helpers ────────────────────────────────────────────────
  function toggleAlbumSelection(id: number) {
    const next = new Set(selectedAlbumIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedAlbumIds = next;
    if (selectedAlbumIds.size < 2) albumMergePicking = false;
  }
  function clearAlbumSelection() {
    selectedAlbumIds = new Set();
    albumMergePicking = false;
  }
  function startAlbumMergePicking() {
    if (selectedAlbumIds.size >= 2) albumMergePicking = true;
  }
  function cancelAlbumMergePicking() {
    albumMergePicking = false;
  }
  async function pickAlbumMergeTarget(targetId: number) {
    if (!albumMergePicking || !selectedAlbumIds.has(targetId)) return;
    const sourceIds = [...selectedAlbumIds].filter(id => id !== targetId);
    const targetTitle = entityCache.getAlbum(targetId)?.title ?? String(targetId);
    const sourceTitles = sourceIds.map(id => entityCache.getAlbum(id)?.title ?? String(id)).join(', ');
    if (!confirm(`Merge "${sourceTitles}" into "${targetTitle}"?\n\nThis cannot be undone.`)) return;
    albumMergeSaving = true;
    try {
      const updated = await mergeAlbums(sourceIds, targetId);
      for (const sid of sourceIds) entityCache.removeAlbum(sid);
      entityCache.upsertAlbum(updated);
      albumIds = albumIds.filter(id => !sourceIds.includes(id));
      albumsTotal = Math.max(0, albumsTotal - sourceIds.length);
      invalidateAlbumNames();
      clearAlbumSelection();
    } catch (e) {
      alert(e instanceof Error ? e.message : String(e));
    } finally {
      albumMergeSaving = false;
    }
  }

  // ── Reference helpers ─────────────────────────────────────────────────────
  async function addReference(
    entity: 'tracks' | 'albums' | 'artists',
    id: number,
    body: AddReferenceBody,
  ): Promise<void> {
    const updatedRefs = await addEntityReference(entity, id, body);
    _applyRefUpdate(entity, id, updatedRefs);
  }

  async function deleteReference(
    entity: 'tracks' | 'albums' | 'artists',
    entityId: number,
    ref: ReferenceDto,
  ): Promise<void> {
    if (ref.id == null) return;
    await deleteEntityReference(entity, entityId, ref.id);
    const current =
      entity === 'tracks' ? entityCache.getTrack(entityId)?.references
        : entity === 'albums' ? entityCache.getAlbum(entityId)?.references
          : entityCache.getArtist(entityId)?.references;
    const updatedRefs = (current ?? []).filter(r => r.id !== ref.id);
    _applyRefUpdate(entity, entityId, updatedRefs);
  }

  function _applyRefUpdate(
    entity: 'tracks' | 'albums' | 'artists',
    id: number,
    refs: ReferenceDto[],
  ): void {
    if (entity === 'tracks') {
      const t = entityCache.getTrack(id);
      if (t) entityCache.upsertTrack({ ...t, references: refs });
      if (editState?.type === 'track' && editState.item.id === id) {
        editState = { ...editState, item: { ...editState.item, references: refs } };
      }
    } else if (entity === 'albums') {
      const a = entityCache.getAlbum(id);
      if (a) entityCache.upsertAlbum({ ...a, references: refs });
      if (editState?.type === 'album' && editState.item.id === id) {
        editState = { ...editState, item: { ...editState.item, references: refs } };
      }
    } else {
      const a = entityCache.getArtist(id);
      if (a) entityCache.upsertArtist({ ...a, references: refs });
      if (editState?.type === 'artist' && editState.item.id === id) {
        editState = { ...editState, item: { ...editState.item, references: refs } };
      }
    }
  }

  // ── Utilities ──────────────────────────────────────────────────────────────
  function fmtDuration(secs: number | null): string {
    if (secs == null) return '\u2014';
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${String(s).padStart(2, '0')}`;
  }
  function isRemote(url: string | null | undefined): boolean {
    return url != null && (url.startsWith('http://') || url.startsWith('https://'));
  }

  // ── Public API ─────────────────────────────────────────────────────────────
  return {
    get tab() { return tab; },
    get tracksView() { return tracksView; }, set tracksView(v: ViewMode) { tracksView = v; },
    get albumsView() { return albumsView; }, set albumsView(v: ViewMode) { albumsView = v; },
    get artistsView() { return artistsView; }, set artistsView(v: ViewMode) { artistsView = v; },

    get artistsSortBy() { return artistsSortBy; },
    set artistsSortBy(v: ArtistSortBy) { artistsSortBy = v; resetArtists(); },
    get artistsSortDir() { return artistsSortDir; },
    set artistsSortDir(v: SortDirection) { artistsSortDir = v; resetArtists(); },
    get albumsSortBy() { return albumsSortBy; },
    set albumsSortBy(v: AlbumSortBy) { albumsSortBy = v; resetAlbums(); },
    get albumsSortDir() { return albumsSortDir; },
    set albumsSortDir(v: SortDirection) { albumsSortDir = v; resetAlbums(); },
    get tracksSortBy() { return tracksSortBy; },
    set tracksSortBy(v: TrackSortBy) { tracksSortBy = v; resetTracks(); },
    get tracksSortDir() { return tracksSortDir; },
    set tracksSortDir(v: SortDirection) { tracksSortDir = v; resetTracks(); },

    get tracks() { return filteredTracks; },
    get tracksLoaded() { return tracksLoaded; },
    get tracksLoading() { return tracksLoading; },
    get tracksLoadingMore() { return tracksLoadingMore; },
    get tracksError() { return tracksError; },
    get tracksTotal() { return tracksTotal; },
    get tracksHasMore() { return tracksHasMore; },
    loadMoreTracks,

    get albums() { return filteredAlbums; },
    get albumsLoaded() { return albumsLoaded; },
    get albumsLoading() { return albumsLoading; },
    get albumsLoadingMore() { return albumsLoadingMore; },
    get albumsError() { return albumsError; },
    get albumsTotal() { return albumsTotal; },
    get albumsHasMore() { return albumsHasMore; },
    loadMoreAlbums,

    get artists() { return filteredArtists; },
    get artistsLoaded() { return artistsLoaded; },
    get artistsLoading() { return artistsLoading; },
    get artistsLoadingMore() { return artistsLoadingMore; },
    get artistsError() { return artistsError; },
    get artistsTotal() { return artistsTotal; },
    get artistsHasMore() { return artistsHasMore; },
    loadMoreArtists,

    get playlists() { return filteredPlaylists; },
    get playlistsLoaded() { return playlistsLoaded; },
    get playlistsLoading() { return playlistsLoading; },
    get playlistsLoadingMore() { return playlistsLoadingMore; },
    get playlistsError() { return playlistsError; },
    get playlistsTotal() { return playlistsTotal; },
    get playlistsHasMore() { return playlistsHasMore; },
    loadMorePlaylists,

    get drillPlaylistId() { return drillPlaylistId; },
    get drillPlaylist() { return drillPlaylist; },
    get drillPlaylistTracks() { return drillPlaylistTracks; },
    get drillPlaylistTracksLoading() { return drillPlaylistTracksLoading; },
    get drillPlaylistTracksError() { return drillPlaylistTracksError; },

    get trackSearch() { return trackSearch; },
    set trackSearch(v: string) { trackSearch = v; debouncedResetTracks(); },
    get albumSearch() { return albumSearch; },
    set albumSearch(v: string) { albumSearch = v; debouncedResetAlbums(); },
    get artistSearch() { return artistSearch; },
    set artistSearch(v: string) { artistSearch = v; debouncedResetArtists(); },
    get playlistSearch() { return playlistSearch; },
    set playlistSearch(v: string) { playlistSearch = v; debouncedResetPlaylists(); },
    get trackFilter() { return trackFilter; },
    set trackFilter(v: TrackFilter) { trackFilter = v; resetTracks(); },

    get drillArtistId() { return drillArtistId; },
    get drillAlbumId() { return drillAlbumId; },
    get drillArtist() { return drillArtist; },
    get drillAlbum() { return drillAlbum; },
    get drillDataLoading() { return drillDataLoading; },
    get artistAlbums() { return artistAlbums; },
    get artistTracks() { return artistTracks; },
    get albumTracks() { return albumTracks; },
    get artistTracksByAlbum() { return artistTracksByAlbum; },
    get filteredTracks() { return filteredTracks; },
    get filteredAlbums() { return filteredAlbums; },
    get filteredArtists() { return filteredArtists; },
    get filteredPlaylists() { return filteredPlaylists; },
    get pendingCount() { return pendingCount; },

    get editState() { return editState; }, set editState(v: EditState) { editState = v; },
    get editSaving() { return editSaving; },
    get imageUploading() { return imageUploading; },
    get thumbnailFetching() { return thumbnailFetching; },
    get trackDraft() { return trackDraft; },
    get albumDraft() { return albumDraft; },
    get artistDraft() { return artistDraft; },

    get batchFetchingArtists() { return batchFetchingArtists; },
    get batchFetchingAlbums() { return batchFetchingAlbums; },
    get batchFetchResult() { return batchFetchResult; },

    get hoveredItem() { return hoveredItem; }, set hoveredItem(v: HoveredItem) { hoveredItem = v; },

    get selectedArtistIds() { return selectedArtistIds; },
    get mergePicking() { return mergePicking; },
    get mergeSaving() { return mergeSaving; },
    get similarFilterActive() { return similarFilterActive; },
    set similarFilterActive(v: boolean) { similarFilterActive = v; if (v) ensureArtistNames(); },
    get similarArtistIds() { return similarArtistIds; },

    get selectedAlbumIds() { return selectedAlbumIds; },
    get albumMergePicking() { return albumMergePicking; },
    get albumMergeSaving() { return albumMergeSaving; },
    get albumSimilarFilterActive() { return albumSimilarFilterActive; },
    set albumSimilarFilterActive(v: boolean) { albumSimilarFilterActive = v; if (v) ensureAlbumNames(); },
    get similarAlbumIds() { return similarAlbumIds; },

    get artistNames() { return artistNames; },
    ensureArtistNames,

    // Cache passthroughs — used by templates that need to resolve an entity by
    // id (e.g. a track's embedded artist/album stub) instead of trusting a
    // possibly-stale snapshot fetched on a different page.
    getTrack: entityCache.getTrack,
    getAlbum: entityCache.getAlbum,
    getArtist: entityCache.getArtist,
    getPlaylist: entityCache.getPlaylist,
    artistDisplayName: entityCache.artistName,
    albumDisplayTitle: entityCache.albumTitle,

    navigate, applyHash, switchTab, clearDrill, handleRefresh, loadAll,
    drillIntoArtist, drillIntoAlbum, backToArtist, backToRoot,
    drillIntoPlaylist,
    startEditTrack, startEditAlbum, startEditArtist,
    openEditForHovered, saveEdit, uploadImage, fetchThumbnailFromReferences,
    batchFetchArtistIconsAction, batchFetchAlbumCoversAction,
    handleDeleteTrack, handleDeleteAlbum, handleDeleteArtist, handleDeletePlaylist,
    toggleArtistSelection, clearArtistSelection, startMergePicking, cancelMergePicking, pickMergeTarget,
    toggleAlbumSelection, clearAlbumSelection, startAlbumMergePicking, cancelAlbumMergePicking, pickAlbumMergeTarget,
    fmtDuration, isRemote,
    addReference, deleteReference,

    get refreshing() { return refreshing; },
    get lastRefreshed() { return lastRefreshed; },
  };
}

export const lib = createLibraryStore();
