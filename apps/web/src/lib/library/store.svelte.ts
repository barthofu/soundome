import {
  getTracks, updateTrack, deleteTrack,
  getAlbums, updateAlbum, deleteAlbum, mergeAlbums,
  getArtists, updateArtist, deleteArtist, mergeArtists,
  uploadArtistImage, uploadAlbumImage, uploadTrackImage,
  fetchArtistIconFromReferences, fetchAlbumCoverFromReferences,
  batchFetchArtistIcons, batchFetchAlbumCovers,
  getPlaylists, getPlaylistTracks, deletePlaylist,
  addEntityReference, deleteEntityReference,
} from '../api';
import type {
  LibraryTrackDto, UpdateTrackBody,
  LibraryAlbumDto, UpdateAlbumBody,
  LibraryArtistDto, UpdateArtistBody,
  LibraryPlaylistDto, PlaylistTrackDto,
  ReferenceDto, AddReferenceBody,
} from '../types';

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

// ── Artist name similarity helpers ────────────────────────────────────────────
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

// ── Sorting helpers ──────────────────────────────────────────────────────────
function sortArtists(list: LibraryArtistDto[], by: ArtistSortBy, dir: SortDirection, tracks: LibraryTrackDto[], albums: LibraryAlbumDto[]): LibraryArtistDto[] {
  const sorted = [...list].sort((a, b) => {
    let cmp = 0;
    if (by === 'name') {
      cmp = a.name.localeCompare(b.name);
    } else if (by === 'track_count') {
      const aCount = tracks.filter(t => t.artists.some(ar => ar.id === a.id)).length;
      const bCount = tracks.filter(t => t.artists.some(ar => ar.id === b.id)).length;
      cmp = aCount - bCount;
    } else if (by === 'album_count') {
      const aCount = albums.filter(al => al.artists.some(ar => ar.id === a.id)).length;
      const bCount = albums.filter(al => al.artists.some(ar => ar.id === b.id)).length;
      cmp = aCount - bCount;
    }
    return dir === 'asc' ? cmp : -cmp;
  });
  return sorted;
}

function sortAlbums(list: LibraryAlbumDto[], by: AlbumSortBy, dir: SortDirection, tracks: LibraryTrackDto[]): LibraryAlbumDto[] {
  const sorted = [...list].sort((a, b) => {
    let cmp = 0;
    if (by === 'title') {
      cmp = a.title.localeCompare(b.title);
    } else if (by === 'date') {
      cmp = (a.date ?? '').localeCompare(b.date ?? '');
    } else if (by === 'artist') {
      const aArtist = a.artists.map(x => x.name).join(', ');
      const bArtist = b.artists.map(x => x.name).join(', ');
      cmp = aArtist.localeCompare(bArtist);
    } else if (by === 'track_count') {
      const aCount = tracks.filter(t => t.album?.id === a.id).length;
      const bCount = tracks.filter(t => t.album?.id === b.id).length;
      cmp = aCount - bCount;
    }
    return dir === 'asc' ? cmp : -cmp;
  });
  return sorted;
}

function sortTracks(list: LibraryTrackDto[], by: TrackSortBy, dir: SortDirection): LibraryTrackDto[] {
  const sorted = [...list].sort((a, b) => {
    let cmp = 0;
    if (by === 'title') {
      cmp = a.title.localeCompare(b.title);
    } else if (by === 'artist') {
      const aArtist = a.artists.map(x => x.name).join(', ');
      const bArtist = b.artists.map(x => x.name).join(', ');
      cmp = aArtist.localeCompare(bArtist);
    } else if (by === 'album') {
      cmp = (a.album?.title ?? '').localeCompare(b.album?.title ?? '');
    } else if (by === 'date') {
      cmp = (a.date ?? '').localeCompare(b.date ?? '');
    } else if (by === 'duration') {
      cmp = (a.duration ?? 0) - (b.duration ?? 0);
    }
    return dir === 'asc' ? cmp : -cmp;
  });
  return sorted;
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

// ── Album title similarity helper (mirrors artist name similarity) ───────────
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

  // Sort state
  let artistsSortBy: ArtistSortBy = $state('name');
  let artistsSortDir: SortDirection = $state('asc');
  let albumsSortBy: AlbumSortBy = $state('title');
  let albumsSortDir: SortDirection = $state('asc');
  let tracksSortBy: TrackSortBy = $state('title');
  let tracksSortDir: SortDirection = $state('asc');

  let tracks: LibraryTrackDto[] = $state([]);
  let tracksLoaded = $state(false);
  let tracksLoading = $state(false);
  let tracksError: string | null = $state(null);

  let albums: LibraryAlbumDto[] = $state([]);
  let albumsLoaded = $state(false);
  let albumsLoading = $state(false);
  let albumsError: string | null = $state(null);

  let artists: LibraryArtistDto[] = $state([]);
  let artistsLoaded = $state(false);
  let artistsLoading = $state(false);
  let artistsError: string | null = $state(null);

  let playlists: LibraryPlaylistDto[] = $state([]);
  let playlistsLoaded = $state(false);
  let playlistsLoading = $state(false);
  let playlistsError: string | null = $state(null);

  let drillPlaylistId: number | null = $state(_initHash.playlistId);
  let drillPlaylistTracks: PlaylistTrackDto[] = $state([]);
  let drillPlaylistTracksLoading = $state(false);
  let drillPlaylistTracksError: string | null = $state(null);

  // ── Global refresh state ───────────────────────────────────────────────────
  let refreshing = $state(false);
  let lastRefreshed: Date | null = $state(null);

  let trackSearch = $state('');
  let albumSearch = $state('');
  let artistSearch = $state('');
  let playlistSearch = $state('');
  let trackFilter: TrackFilter = $state('ok');

  let drillArtistId: number | null = $state(_initHash.artistId);
  let drillAlbumId: number | null = $state(_initHash.albumId);

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

  // ── Derived ────────────────────────────────────────────────────────────────
  let drillArtist = $derived(
    drillArtistId != null ? (artists.find(a => a.id === drillArtistId) ?? null) : null
  );
  let drillAlbum = $derived(
    drillAlbumId != null ? (albums.find(a => a.id === drillAlbumId) ?? null) : null
  );
  let drillPlaylist = $derived(
    drillPlaylistId != null ? (playlists.find(p => p.id === drillPlaylistId) ?? null) : null
  );
  let artistAlbums = $derived.by(() => {
    const d = drillArtist; if (!d) return [];
    return albums.filter(a => a.artists.some(ar => ar.id === d.id));
  });
  let artistTracks = $derived.by(() => {
    const d = drillArtist; if (!d) return [];
    return tracks.filter(t => t.artists.some(a => a.id === d.id));
  });
  let albumTracks = $derived.by(() => {
    const d = drillAlbum; if (!d) return [];
    return tracks.filter(t => t.album?.id === d.id);
  });
  let artistTracksByAlbum = $derived.by(() => {
    if (!drillArtist) return [];
    type Group = { albumId: number | null; albumTitle: string | null; albumCover: string | null; tracks: LibraryTrackDto[] };
    const map = new Map<string, Group>();
    for (const t of artistTracks) {
      const key = t.album?.id != null ? String(t.album.id) : '__none__';
      if (!map.has(key)) map.set(key, { albumId: t.album?.id ?? null, albumTitle: t.album?.title ?? null, albumCover: null, tracks: [] });
      map.get(key)!.tracks.push(t);
    }
    const result: Group[] = [];
    for (const [, grp] of map) {
      const fullAlbum = grp.albumId != null ? albums.find(a => a.id === grp.albumId) : null;
      result.push({ ...grp, albumCover: fullAlbum?.cover ?? null });
    }
    result.sort((a, b) => {
      if (a.albumId === null) return 1;
      if (b.albumId === null) return -1;
      const aDate = albums.find(x => x.id === a.albumId)?.date ?? '';
      const bDate = albums.find(x => x.id === b.albumId)?.date ?? '';
      if (aDate !== bDate) return aDate < bDate ? -1 : 1;
      return (a.albumTitle ?? '') < (b.albumTitle ?? '') ? -1 : 1;
    });
    return result;
  });
  let filteredTracks = $derived.by(() => {
    let list = tracks;
    const q = trackSearch.trim().toLowerCase();
    if (q) list = list.filter(t => t.title.toLowerCase().includes(q) || t.artists.some(a => a.name.toLowerCase().includes(q)));
    if (trackFilter === 'ok') list = list.filter(t => !t.needs_validation);
    if (trackFilter === 'pending') list = list.filter(t => t.needs_validation);
    return sortTracks(list, tracksSortBy, tracksSortDir);
  });
  let filteredAlbums = $derived.by(() => {
    const q = albumSearch.trim().toLowerCase();
    let list = !q ? albums : albums.filter(a => a.title.toLowerCase().includes(q) || a.artists.some(ar => ar.name.toLowerCase().includes(q)));
    return sortAlbums(list, albumsSortBy, albumsSortDir, tracks);
  });
  let filteredArtists = $derived.by(() => {
    const q = artistSearch.trim().toLowerCase();
    let list = !q ? artists : artists.filter(a => a.name.toLowerCase().includes(q));
    return sortArtists(list, artistsSortBy, artistsSortDir, tracks, albums);
  });
  let filteredPlaylists = $derived.by(() => {
    const q = playlistSearch.trim().toLowerCase(); if (!q) return playlists;
    return playlists.filter(p => p.name.toLowerCase().includes(q));
  });
  let pendingCount = $derived(tracks.filter(t => t.needs_validation).length);
  let similarArtistIds = $derived.by(() => {
    const ids = new Set<number>();
    for (let i = 0; i < artists.length; i++) {
      for (let j = i + 1; j < artists.length; j++) {
        if (areSimilarArtistNames(artists[i].name, artists[j].name)) {
          ids.add(artists[i].id);
          ids.add(artists[j].id);
        }
      }
    }
    return ids;
  });
  let similarAlbumIds = $derived.by(() => {
    const ids = new Set<number>();
    for (let i = 0; i < albums.length; i++) {
      for (let j = i + 1; j < albums.length; j++) {
        if (areSimilarAlbumNames(albums[i].title, albums[j].title)) {
          ids.add(albums[i].id);
          ids.add(albums[j].id);
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
      drillPlaylistId = p[1] ? (parseInt(p[1]) || null) : null; return;
    }
    if (t === 'albums') { tab = 'albums'; drillArtistId = null; drillAlbumId = p[1] ? (parseInt(p[1]) || null) : null; drillPlaylistId = null; return; }
    tab = 'artists';
    drillArtistId = p[1] ? (parseInt(p[1]) || null) : null;
    drillAlbumId = (p[2] === 'album' && p[3]) ? (parseInt(p[3]) || null) : null;
    drillPlaylistId = null;
  }
  function switchTab(t: Tab) { navigate(t); clearArtistSelection(); clearAlbumSelection(); }
  function clearDrill() { navigate(tab); }

  function handleRefresh() {
    clearDrill();
    loadAll();
  }

  async function loadAll() {
    refreshing = true;
    tracksLoaded = false; albumsLoaded = false; artistsLoaded = false; playlistsLoaded = false;
    tracks = []; albums = []; artists = []; playlists = [];
    try {
      await Promise.all([loadTracks(), loadAlbums(), loadArtists(), loadPlaylists()]);
      lastRefreshed = new Date();
    } finally {
      refreshing = false;
    }
  }

  function drillIntoArtist(a: LibraryArtistDto) { navigate('artists', a.id); }
  function drillIntoAlbum(album: LibraryAlbumDto) {
    if (tab === 'albums') navigate('albums', undefined, album.id);
    else navigate('artists', drillArtistId ?? undefined, album.id);
  }
  function backToArtist() { if (drillArtistId) navigate('artists', drillArtistId); }
  function backToRoot() { navigate(tab); }

  // ── Data loading ───────────────────────────────────────────────────────────
  async function loadTracks() {
    tracksLoading = true; tracksError = null;
    try { tracks = await getTracks(); tracksLoaded = true; }
    catch (e) { tracksError = e instanceof Error ? e.message : String(e); tracksLoaded = true; }
    finally { tracksLoading = false; }
  }
  async function loadAlbums() {
    albumsLoading = true; albumsError = null;
    try { albums = await getAlbums(); albumsLoaded = true; }
    catch (e) { albumsError = e instanceof Error ? e.message : String(e); albumsLoaded = true; }
    finally { albumsLoading = false; }
  }
  async function loadArtists() {
    artistsLoading = true; artistsError = null;
    try { artists = await getArtists(); artistsLoaded = true; }
    catch (e) { artistsError = e instanceof Error ? e.message : String(e); artistsLoaded = true; }
    finally { artistsLoading = false; }
  }
  async function loadPlaylists() {
    playlistsLoading = true; playlistsError = null;
    try { playlists = await getPlaylists(); playlistsLoaded = true; }
    catch (e) { playlistsError = e instanceof Error ? e.message : String(e); playlistsLoaded = true; }
    finally { playlistsLoading = false; }
  }
  async function loadDrillPlaylistTracks(id: number) {
    drillPlaylistTracksLoading = true; drillPlaylistTracksError = null;
    try { drillPlaylistTracks = await getPlaylistTracks(id); }
    catch (e) { drillPlaylistTracksError = e instanceof Error ? e.message : String(e); }
    finally { drillPlaylistTracksLoading = false; }
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
    if (hoveredItem.type === 'track') { const t = tracks.find(x => x.id === hoveredItem!.id); if (t) startEditTrack(t); }
    else if (hoveredItem.type === 'album') { const a = albums.find(x => x.id === hoveredItem!.id); if (a) startEditAlbum(a); }
    else { const a = artists.find(x => x.id === hoveredItem!.id); if (a) startEditArtist(a); }
  }
  async function saveEdit() {
    if (!editState) return;
    const state = editState;
    editSaving = true;
    try {
      if (state.type === 'track') {
        const updated = await updateTrack(state.item.id, trackDraft);
        tracks = tracks.map(t => t.id === state.item.id ? updated : t);
      } else if (state.type === 'album') {
        const updated = await updateAlbum(state.item.id, albumDraft);
        albums = albums.map(a => a.id === state.item.id ? updated : a);
      } else {
        const updated = await updateArtist(state.item.id, artistDraft);
        artists = artists.map(a => a.id === state.item.id ? updated : a);
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
        artists = artists.map(a => a.id === state.item.id ? updated : a);
        editState = { type: 'artist', item: updated };
        artistDraft.icon = url;
      } else if (state.type === 'album') {
        const { url } = await uploadAlbumImage(state.item.id, file);
        const updated = { ...state.item, cover: url };
        albums = albums.map(a => a.id === state.item.id ? updated : a);
        editState = { type: 'album', item: updated };
        albumDraft.cover = url;
      } else {
        const { url } = await uploadTrackImage(state.item.id, file);
        const updated = { ...state.item, cover: url };
        tracks = tracks.map(t => t.id === state.item.id ? updated : t);
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
   * Only supported for artists (icon) and albums (cover) — the button that triggers
   * this is hidden for tracks in the edit modal.
   */
  async function fetchThumbnailFromReferences() {
    if (!editState) return;
    const state = editState;
    thumbnailFetching = true;
    try {
      if (state.type === 'artist') {
        const { url } = await fetchArtistIconFromReferences(state.item.id);
        const updated = { ...state.item, icon: url };
        artists = artists.map(a => a.id === state.item.id ? updated : a);
        editState = { type: 'artist', item: updated };
        artistDraft.icon = url;
      } else if (state.type === 'album') {
        const { url } = await fetchAlbumCoverFromReferences(state.item.id);
        const updated = { ...state.item, cover: url };
        albums = albums.map(a => a.id === state.item.id ? updated : a);
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
      // Refresh artists list to reflect the batch updates
      await loadArtists();
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
      // Refresh albums list to reflect the batch updates
      await loadAlbums();
    } catch (err) {
      alert(err instanceof Error ? err.message : String(err));
    } finally {
      batchFetchingAlbums = false;
    }
  }

  // ── Delete handlers ────────────────────────────────────────────────────────
  async function handleDeleteTrack(id: number) {
    if (!confirm('Delete this track from the library?')) return;
    try { await deleteTrack(id); tracks = tracks.filter(t => t.id !== id); }
    catch (e) { alert(e instanceof Error ? e.message : String(e)); }
  }
  async function handleDeleteAlbum(id: number) {
    if (!confirm('Delete this album? Tracks will remain but lose their album association.')) return;
    try {
      await deleteAlbum(id); albums = albums.filter(a => a.id !== id);
      if (drillAlbumId === id) navigate(tab, drillArtistId ?? undefined);
    } catch (e) { alert(e instanceof Error ? e.message : String(e)); }
  }
  async function handleDeleteArtist(id: number) {
    if (!confirm('Delete this artist?')) return;
    try {
      await deleteArtist(id); artists = artists.filter(a => a.id !== id);
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
      // If we need to remove tracks from local state but don't have them loaded yet,
      // fetch the list before deleting so we know which IDs to purge.
      let trackIdsToRemove: Set<number> = new Set();
      if (deleteTracks) {
        const source =
          drillPlaylistId === id && drillPlaylistTracks.length > 0
            ? drillPlaylistTracks
            : await getPlaylistTracks(id);
        trackIdsToRemove = new Set(source.map(t => t.id));
      }

      await deletePlaylist(id, deleteTracks);
      playlists = playlists.filter(p => p.id !== id);

      if (deleteTracks && trackIdsToRemove.size > 0) {
        tracks = tracks.filter(t => !trackIdsToRemove.has(t.id));
      }

      if (drillPlaylistId === id) navigate('playlists');
      
      // Refresh all data after deletion
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
    const targetName = artists.find(a => a.id === targetId)?.name ?? String(targetId);
    const sourceNames = sourceIds.map(id => artists.find(a => a.id === id)?.name ?? String(id)).join(', ');
    if (!confirm(`Merge "${sourceNames}" into "${targetName}"?\n\nThis cannot be undone.`)) return;
    mergeSaving = true;
    try {
      const updated = await mergeArtists(sourceIds, targetId);
      artists = artists
        .filter(a => !sourceIds.includes(a.id))
        .map(a => a.id === updated.id ? updated : a);
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
    const targetTitle = albums.find(a => a.id === targetId)?.title ?? String(targetId);
    const sourceTitles = sourceIds.map(id => albums.find(a => a.id === id)?.title ?? String(id)).join(', ');
    if (!confirm(`Merge "${sourceTitles}" into "${targetTitle}"?\n\nThis cannot be undone.`)) return;
    albumMergeSaving = true;
    try {
      const updated = await mergeAlbums(sourceIds, targetId);
      albums = albums
        .filter(a => !sourceIds.includes(a.id))
        .map(a => a.id === updated.id ? updated : a);
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
    const updatedRefs = (entity === 'tracks'
      ? tracks.find(t => t.id === entityId)?.references
      : entity === 'albums'
        ? albums.find(a => a.id === entityId)?.references
        : artists.find(a => a.id === entityId)?.references
    )?.filter(r => r.id !== ref.id) ?? [];
    _applyRefUpdate(entity, entityId, updatedRefs);
  }

  function _applyRefUpdate(
    entity: 'tracks' | 'albums' | 'artists',
    id: number,
    refs: ReferenceDto[],
  ): void {
    if (entity === 'tracks') {
      tracks = tracks.map(t => t.id === id ? { ...t, references: refs } : t);
      if (editState?.type === 'track' && editState.item.id === id) {
        editState = { ...editState, item: { ...editState.item, references: refs } };
      }
    } else if (entity === 'albums') {
      albums = albums.map(a => a.id === id ? { ...a, references: refs } : a);
      if (editState?.type === 'album' && editState.item.id === id) {
        editState = { ...editState, item: { ...editState.item, references: refs } };
      }
    } else {
      artists = artists.map(a => a.id === id ? { ...a, references: refs } : a);
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

    get artistsSortBy() { return artistsSortBy; }, set artistsSortBy(v: ArtistSortBy) { artistsSortBy = v; },
    get artistsSortDir() { return artistsSortDir; }, set artistsSortDir(v: SortDirection) { artistsSortDir = v; },
    get albumsSortBy() { return albumsSortBy; }, set albumsSortBy(v: AlbumSortBy) { albumsSortBy = v; },
    get albumsSortDir() { return albumsSortDir; }, set albumsSortDir(v: SortDirection) { albumsSortDir = v; },
    get tracksSortBy() { return tracksSortBy; }, set tracksSortBy(v: TrackSortBy) { tracksSortBy = v; },
    get tracksSortDir() { return tracksSortDir; }, set tracksSortDir(v: SortDirection) { tracksSortDir = v; },

    get tracks() { return tracks; }, set tracks(v: LibraryTrackDto[]) { tracks = v; },
    get tracksLoaded() { return tracksLoaded; },
    get tracksLoading() { return tracksLoading; },
    get tracksError() { return tracksError; },

    get albums() { return albums; }, set albums(v: LibraryAlbumDto[]) { albums = v; },
    get albumsLoaded() { return albumsLoaded; },
    get albumsLoading() { return albumsLoading; },
    get albumsError() { return albumsError; },

    get artists() { return artists; }, set artists(v: LibraryArtistDto[]) { artists = v; },
    get artistsLoaded() { return artistsLoaded; },
    get artistsLoading() { return artistsLoading; },
    get artistsError() { return artistsError; },

    get playlists() { return playlists; }, set playlists(v: LibraryPlaylistDto[]) { playlists = v; },
    get playlistsLoaded() { return playlistsLoaded; },
    get playlistsLoading() { return playlistsLoading; },
    get playlistsError() { return playlistsError; },

    get drillPlaylistId() { return drillPlaylistId; },
    get drillPlaylist() { return drillPlaylist; },
    get drillPlaylistTracks() { return drillPlaylistTracks; },
    get drillPlaylistTracksLoading() { return drillPlaylistTracksLoading; },
    get drillPlaylistTracksError() { return drillPlaylistTracksError; },

    get trackSearch() { return trackSearch; }, set trackSearch(v: string) { trackSearch = v; },
    get albumSearch() { return albumSearch; }, set albumSearch(v: string) { albumSearch = v; },
    get artistSearch() { return artistSearch; }, set artistSearch(v: string) { artistSearch = v; },
    get playlistSearch() { return playlistSearch; }, set playlistSearch(v: string) { playlistSearch = v; },
    get trackFilter() { return trackFilter; }, set trackFilter(v: TrackFilter) { trackFilter = v; },

    get drillArtistId() { return drillArtistId; },
    get drillAlbumId() { return drillAlbumId; },
    get drillArtist() { return drillArtist; },
    get drillAlbum() { return drillAlbum; },
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
    get similarFilterActive() { return similarFilterActive; }, set similarFilterActive(v: boolean) { similarFilterActive = v; },
    get similarArtistIds() { return similarArtistIds; },

    get selectedAlbumIds() { return selectedAlbumIds; },
    get albumMergePicking() { return albumMergePicking; },
    get albumMergeSaving() { return albumMergeSaving; },
    get albumSimilarFilterActive() { return albumSimilarFilterActive; }, set albumSimilarFilterActive(v: boolean) { albumSimilarFilterActive = v; },
    get similarAlbumIds() { return similarAlbumIds; },

    navigate, applyHash, switchTab, clearDrill, handleRefresh, loadAll,
    drillIntoArtist, drillIntoAlbum, backToArtist, backToRoot,
    drillIntoPlaylist,
    loadTracks, loadAlbums, loadArtists, loadPlaylists,
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
