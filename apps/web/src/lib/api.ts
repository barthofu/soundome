import type {
  PendingValidationDto,
  PatchValidationBody,
  MatchCandidateDto,
  TaskDto,
  LibraryTrackDto,
  UpdateTrackBody,
  LibraryAlbumDto,
  UpdateAlbumBody,
  LibraryArtistDto,
  UpdateArtistBody,
  LibraryPlaylistDto,
  PlaylistTrackDto,
  ReferenceDto,
  AddReferenceBody,
  StructuralFindingDto,
  DuplicateEntityType,
  DuplicateGroupDto,
  DedupIgnoreDto,
} from './types';

const BASE = '/api';

export async function getPendingValidations(): Promise<PendingValidationDto[]> {
  const res = await fetch(`${BASE}/validations`);
  if (!res.ok) throw new Error(`Failed to fetch validations: ${res.statusText}`);
  return res.json();
}

export async function getPendingCount(): Promise<number> {
  const res = await fetch(`${BASE}/validations/count`);
  if (!res.ok) throw new Error(`Failed to fetch validations count: ${res.statusText}`);
  const data: { count: number } = await res.json();
  return data.count;
}

export async function approveValidation(
  id: number,
  patch: PatchValidationBody,
): Promise<PendingValidationDto> {
  const res = await fetch(`${BASE}/validations/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(patch),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(body.message ?? res.statusText);
  }
  return res.json();
}

export async function rejectValidation(id: number): Promise<void> {
  const res = await fetch(`${BASE}/validations/${id}`, { method: 'DELETE' });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(body.message ?? res.statusText);
  }
}

export async function getMatchCandidates(id: number): Promise<MatchCandidateDto[]> {
  const res = await fetch(`${BASE}/validations/${id}/matches`);
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(body.message ?? res.statusText);
  }
  return res.json();
}

export async function getYoutubeCandidates(id: number): Promise<MatchCandidateDto[]> {
  const res = await fetch(`${BASE}/validations/${id}/youtube-candidates`);
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(body.message ?? res.statusText);
  }
  return res.json();
}

export type DownloadResultTrack = {
  type: 'track';
  title: string;
  artists: string[];
  needs_validation: boolean;
};

export type DownloadResultPlaylist = {
  type: 'playlist';
  task_id: number;
};

export type DownloadResult = DownloadResultTrack | DownloadResultPlaylist;

export async function downloadUrl(url: string): Promise<DownloadResult> {
  const res = await fetch(`${BASE}/download`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ url }),
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(body.message ?? res.statusText);
  }
  return res.json();
}

export type RecentTrack = {
  id: number;
  title: string;
  artists: { id: number | null; name: string }[];
  album: { id: number | null; title: string } | null;
  cover: string | null;
  duration: number | null;
  needs_validation: boolean;
  validation_reason: string | null;
};

export async function getProviders(): Promise<string[]> {
  const res = await fetch(`${BASE}/providers`);
  if (!res.ok) return [];
  const data: { providers: string[] } = await res.json();
  return data.providers;
}

export async function getRecentTracks(limit = 20): Promise<RecentTrack[]> {
  const res = await fetch(`${BASE}/tracks/recent?limit=${limit}`);
  if (!res.ok) throw new Error(`Failed to fetch recent tracks: ${res.statusText}`);
  return res.json();
}

export async function getTasks(): Promise<TaskDto[]> {
  const res = await fetch(`${BASE}/tasks`);
  if (!res.ok) throw new Error(`Failed to fetch tasks: ${res.statusText}`);
  return res.json();
}

export async function retryTask(id: number): Promise<TaskDto> {
  const res = await fetch(`${BASE}/tasks/${id}/retry`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function cancelTask(id: number): Promise<TaskDto> {
  const res = await fetch(`${BASE}/tasks/${id}/cancel`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function getActiveTasksCount(): Promise<number> {
  const res = await fetch(`${BASE}/tasks/active-count`);
  if (!res.ok) throw new Error(`Failed to fetch active tasks count: ${res.statusText}`);
  const data: { count: number } = await res.json();
  return data.count;
}

// ================================================================================================
// Library — Tracks
// ================================================================================================

export interface PageResult<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface TrackListParams {
  page?: number;
  pageSize?: number;
  q?: string;
  sortBy?: string;
  sortDir?: 'asc' | 'desc';
  filter?: 'all' | 'ok' | 'pending';
}

function buildQuery(params: Record<string, string | number | undefined>): string {
  const usp = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== '') usp.set(k, String(v));
  }
  const s = usp.toString();
  return s ? `?${s}` : '';
}

export async function getTracksPage(params: TrackListParams = {}): Promise<PageResult<LibraryTrackDto>> {
  const qs = buildQuery({
    page: params.page,
    page_size: params.pageSize,
    q: params.q,
    sort_by: params.sortBy,
    sort_dir: params.sortDir,
    filter: params.filter,
  });
  const res = await fetch(`${BASE}/tracks${qs}`);
  if (!res.ok) throw new Error(`Failed to fetch tracks: ${res.statusText}`);
  return res.json();
}

export async function updateTrack(id: number, body: UpdateTrackBody): Promise<LibraryTrackDto> {
  const res = await fetch(`${BASE}/tracks/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function deleteTrack(id: number): Promise<void> {
  const res = await fetch(`${BASE}/tracks/${id}`, { method: 'DELETE' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

// ================================================================================================
// Library — Albums
// ================================================================================================

export interface AlbumListParams {
  page?: number;
  pageSize?: number;
  q?: string;
  sortBy?: string;
  sortDir?: 'asc' | 'desc';
}

export async function getAlbumsPage(params: AlbumListParams = {}): Promise<PageResult<LibraryAlbumDto>> {
  const qs = buildQuery({
    page: params.page,
    page_size: params.pageSize,
    q: params.q,
    sort_by: params.sortBy,
    sort_dir: params.sortDir,
  });
  const res = await fetch(`${BASE}/albums${qs}`);
  if (!res.ok) throw new Error(`Failed to fetch albums: ${res.statusText}`);
  return res.json();
}

/** Lightweight `{id, title}` pairs for every album — used by the duplicate-detection workflow. */
export async function getAlbumNames(): Promise<{ id: number; title: string }[]> {
  const res = await fetch(`${BASE}/albums/names`);
  if (!res.ok) throw new Error(`Failed to fetch album names: ${res.statusText}`);
  return res.json();
}

export async function getAlbumTracks(id: number): Promise<LibraryTrackDto[]> {
  const res = await fetch(`${BASE}/albums/${id}/tracks`);
  if (!res.ok) throw new Error(`Failed to fetch album tracks: ${res.statusText}`);
  return res.json();
}

export async function updateAlbum(id: number, body: UpdateAlbumBody): Promise<LibraryAlbumDto> {
  const res = await fetch(`${BASE}/albums/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function deleteAlbum(id: number): Promise<void> {
  const res = await fetch(`${BASE}/albums/${id}`, { method: 'DELETE' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

export async function mergeAlbums(
  sourceIds: number[],
  targetId: number,
): Promise<LibraryAlbumDto> {
  const res = await fetch(`${BASE}/albums/merge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ source_ids: sourceIds, target_id: targetId }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

// ================================================================================================
// Library — Artists
// ================================================================================================

export interface ArtistListParams {
  page?: number;
  pageSize?: number;
  q?: string;
  sortBy?: string;
  sortDir?: 'asc' | 'desc';
}

export async function getArtistsPage(params: ArtistListParams = {}): Promise<PageResult<LibraryArtistDto>> {
  const qs = buildQuery({
    page: params.page,
    page_size: params.pageSize,
    q: params.q,
    sort_by: params.sortBy,
    sort_dir: params.sortDir,
  });
  const res = await fetch(`${BASE}/artists${qs}`);
  if (!res.ok) throw new Error(`Failed to fetch artists: ${res.statusText}`);
  return res.json();
}

/** Lightweight `{id, name}` pairs for every artist — used by the duplicate-detection workflow. */
export async function getArtistNames(): Promise<{ id: number; name: string }[]> {
  const res = await fetch(`${BASE}/artists/names`);
  if (!res.ok) throw new Error(`Failed to fetch artist names: ${res.statusText}`);
  return res.json();
}

export async function getArtistTracks(id: number): Promise<LibraryTrackDto[]> {
  const res = await fetch(`${BASE}/artists/${id}/tracks`);
  if (!res.ok) throw new Error(`Failed to fetch artist tracks: ${res.statusText}`);
  return res.json();
}

export async function getArtistAlbums(id: number): Promise<LibraryAlbumDto[]> {
  const res = await fetch(`${BASE}/artists/${id}/albums`);
  if (!res.ok) throw new Error(`Failed to fetch artist albums: ${res.statusText}`);
  return res.json();
}

export async function updateArtist(id: number, body: UpdateArtistBody): Promise<LibraryArtistDto> {
  const res = await fetch(`${BASE}/artists/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function deleteArtist(id: number): Promise<void> {
  const res = await fetch(`${BASE}/artists/${id}`, { method: 'DELETE' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

export async function mergeArtists(
  sourceIds: number[],
  targetId: number,
): Promise<LibraryArtistDto> {
  const res = await fetch(`${BASE}/artists/merge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ source_ids: sourceIds, target_id: targetId }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

// ================================================================================================
// Image uploads
// ================================================================================================

export interface ImageResponse {
  url: string;
}

async function uploadImage(endpoint: string, file: File): Promise<ImageResponse> {
  const form = new FormData();
  form.append('file', file);
  const res = await fetch(endpoint, { method: 'POST', body: form });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function uploadArtistImage(id: number, file: File): Promise<ImageResponse> {
  return uploadImage(`${BASE}/artists/${id}/image`, file);
}

export async function uploadAlbumImage(id: number, file: File): Promise<ImageResponse> {
  return uploadImage(`${BASE}/albums/${id}/image`, file);
}

export async function uploadTrackImage(id: number, file: File): Promise<ImageResponse> {
  return uploadImage(`${BASE}/tracks/${id}/image`, file);
}

/**
 * Best-effort: resolve an artist's photo from its existing references (Spotify,
 * SoundCloud, YouTube Music) and persist it as the artist's icon.
 * Throws when no reference resolves to an image (404) or on network/DB error.
 */
export async function fetchArtistIconFromReferences(id: number): Promise<ImageResponse> {
  const res = await fetch(`${BASE}/artists/${id}/fetch-icon`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

/**
 * Best-effort: resolve an album's cover from its existing references (Spotify,
 * SoundCloud, YouTube Music) and persist it as the album's cover.
 * Throws when no reference resolves to an image (404) or on network/DB error.
 */
export async function fetchAlbumCoverFromReferences(id: number): Promise<ImageResponse> {
  const res = await fetch(`${BASE}/albums/${id}/fetch-cover`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export type BatchThumbnailResult = {
  count: number;
  skipped: number;
};

/**
 * Batch fetch: for each artist without an icon, try to resolve one from its
 * existing references (Spotify, SoundCloud, YouTube Music).
 * Returns the number of artists now with an icon and the number that remain without.
 */
export async function batchFetchArtistIcons(): Promise<BatchThumbnailResult> {
  const res = await fetch(`${BASE}/batch/fetch-artist-icons`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

/**
 * Batch fetch: for each album without a cover, try to resolve one from its
 * existing references (Spotify, SoundCloud, YouTube Music).
 * Returns the number of albums now with a cover and the number that remain without.
 */
export async function batchFetchAlbumCovers(): Promise<BatchThumbnailResult> {
  const res = await fetch(`${BASE}/batch/fetch-album-covers`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

// ================================================================================================
// Library — Playlists
// ================================================================================================

export async function getPlaylistsPage(params: { page?: number; pageSize?: number; q?: string } = {}): Promise<PageResult<LibraryPlaylistDto>> {
  const qs = buildQuery({ page: params.page, page_size: params.pageSize, q: params.q });
  const res = await fetch(`${BASE}/playlists${qs}`);
  if (!res.ok) throw new Error(`Failed to fetch playlists: ${res.statusText}`);
  return res.json();
}

export async function getPlaylistTracks(id: number): Promise<PlaylistTrackDto[]> {
  const res = await fetch(`${BASE}/playlists/${id}/tracks`);
  if (!res.ok) throw new Error(`Failed to fetch playlist tracks: ${res.statusText}`);
  return res.json();
}

export async function deletePlaylist(id: number, deleteTracks = false): Promise<void> {
  const url = deleteTracks ? `${BASE}/playlists/${id}?delete_tracks=true` : `${BASE}/playlists/${id}`;
  const res = await fetch(url, { method: 'DELETE' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

// ================================================================================================
// References (tracks, albums, artists)
// ================================================================================================

export async function getEntityReferences(
  entity: 'tracks' | 'albums' | 'artists',
  id: number,
): Promise<ReferenceDto[]> {
  const res = await fetch(`${BASE}/${entity}/${id}/references`);
  if (!res.ok) throw new Error(`Failed to fetch references: ${res.statusText}`);
  return res.json();
}

export async function addEntityReference(
  entity: 'tracks' | 'albums' | 'artists',
  id: number,
  body: AddReferenceBody,
): Promise<ReferenceDto[]> {
  const res = await fetch(`${BASE}/${entity}/${id}/references`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function deleteEntityReference(
  entity: 'tracks' | 'albums' | 'artists',
  entityId: number,
  refId: number,
): Promise<void> {
  const res = await fetch(`${BASE}/${entity}/${entityId}/references/${refId}`, {
    method: 'DELETE',
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

// ================================================================================================
// Sync Schedules (subscriptions) + global sync settings
// ================================================================================================

export type SyncEntityType = 'playlist' | 'artist';

export interface SyncScheduleDto {
  id: number;
  entity_type: SyncEntityType;
  artist_id: number | null;
  reference_id: number | null;
  url: string;
  label: string | null;
  enabled: boolean;
  last_run: string | null;
  created_at: string | null;
}

export async function getSyncSchedules(): Promise<SyncScheduleDto[]> {
  const res = await fetch(`${BASE}/sync-schedules`);
  if (!res.ok) throw new Error(`Failed to fetch sync schedules: ${res.statusText}`);
  return res.json();
}

export async function createSyncSchedule(
  body:
    | { url: string; label?: string | null }
    | { artist_id: number; reference_id: number; label?: string | null },
): Promise<SyncScheduleDto> {
  const res = await fetch(`${BASE}/sync-schedules`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function updateSyncSchedule(
  id: number,
  patch: { label?: string; enabled?: boolean },
): Promise<SyncScheduleDto> {
  const res = await fetch(`${BASE}/sync-schedules/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(patch),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function deleteSyncSchedule(id: number): Promise<void> {
  const res = await fetch(`${BASE}/sync-schedules/${id}`, { method: 'DELETE' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

export async function triggerSyncSchedule(id: number): Promise<{ task_id: number }> {
  const res = await fetch(`${BASE}/sync-schedules/${id}/trigger`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export interface SyncSettingsDto {
  cron_expression: string;
  enabled: boolean;
  last_run: string | null;
  next_run: string | null;
}

export async function getSyncSettings(): Promise<SyncSettingsDto> {
  const res = await fetch(`${BASE}/sync-settings`);
  if (!res.ok) throw new Error(`Failed to fetch sync settings: ${res.statusText}`);
  return res.json();
}

export async function updateSyncSettings(
  patch: { cron_expression?: string; enabled?: boolean },
): Promise<SyncSettingsDto> {
  const res = await fetch(`${BASE}/sync-settings`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(patch),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function triggerAllSyncSchedules(): Promise<{ task_ids: number[] }> {
  const res = await fetch(`${BASE}/sync-settings/trigger`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}



// ================================================================================================
// Ingest
// ================================================================================================

export interface IngestFileTags {
  title: string | null;
  artists: string[];
  album: string | null;
  date: string | null;
  genre: string | null;
  duration_secs: number | null;
  track_number: number | null;
}

export interface IngestFileEntry {
  name: string;
  path: string;
  relative_path: string;
  size_bytes: number;
  tags: IngestFileTags | null;
}

export interface IngestFilesResponse {
  ingest_dir: string;
  files: IngestFileEntry[];
}

export interface IngestResult {
  title: string;
  artists: string[];
  needs_validation: boolean;
}

export async function listIngestFiles(): Promise<IngestFilesResponse> {
  const res = await fetch(`${BASE}/library/ingest/files`);
  if (!res.ok) throw new Error(`Failed to list ingest files: ${res.statusText}`);
  return res.json();
}

export async function ingestFile(filePath: string): Promise<IngestResult> {
  const res = await fetch(`${BASE}/library/ingest`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ file_path: filePath }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function ingestAll(): Promise<{ task_id: number }> {
  const res = await fetch(`${BASE}/library/ingest/all`, { method: 'POST' });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

// ================================================================================================
// Storage Stats
// ================================================================================================

export interface ArtistStorageDto {
  id: number;
  name: string;
  bytes: number;
  percent: number;
}

export interface StorageStatsDto {
  total_bytes: number;
  total_formatted: string;
  artists: ArtistStorageDto[];
}

export async function getStorageStats(): Promise<StorageStatsDto> {
  const res = await fetch(`${BASE}/library/storage-stats`);
  if (!res.ok) throw new Error(`Failed to fetch storage stats: ${res.statusText}`);
  return res.json();
}

export async function getVersion(): Promise<string> {
  const res = await fetch(`${BASE}/version`);
  if (!res.ok) return '';
  const data: { version: string } = await res.json();
  return data.version;
}

// ── Data Quality ─────────────────────────────────────────────────────────────

export async function getStructuralFindings(): Promise<StructuralFindingDto[]> {
  const res = await fetch(`${BASE}/data-quality/audit/structural`);
  if (!res.ok) throw new Error(`Failed to fetch structural findings: ${res.statusText}`);
  return res.json();
}

export async function getDuplicateGroups(
  entityType: DuplicateEntityType,
): Promise<DuplicateGroupDto[]> {
  const res = await fetch(`${BASE}/data-quality/duplicates/${entityType}`);
  if (!res.ok) throw new Error(`Failed to fetch ${entityType} duplicates: ${res.statusText}`);
  return res.json();
}

export async function getIgnoredDuplicates(
  entityType: DuplicateEntityType,
): Promise<DedupIgnoreDto[]> {
  const res = await fetch(`${BASE}/data-quality/duplicates/${entityType}/ignored`);
  if (!res.ok) throw new Error(`Failed to fetch ignored ${entityType} pairs: ${res.statusText}`);
  return res.json();
}

export async function ignoreDuplicatePair(
  entityType: DuplicateEntityType,
  idA: number,
  idB: number,
): Promise<void> {
  const res = await fetch(`${BASE}/data-quality/duplicates/${entityType}/ignore`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id_a: idA, id_b: idB }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

export async function restoreIgnoredDuplicate(
  entityType: DuplicateEntityType,
  idA: number,
  idB: number,
): Promise<void> {
  const res = await fetch(
    `${BASE}/data-quality/duplicates/${entityType}/ignore/${idA}/${idB}`,
    { method: 'DELETE' },
  );
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
}

export async function mergeTracks(sourceIds: number[], targetId: number): Promise<LibraryTrackDto> {
  const res = await fetch(`${BASE}/tracks/merge`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ source_ids: sourceIds, target_id: targetId }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}
