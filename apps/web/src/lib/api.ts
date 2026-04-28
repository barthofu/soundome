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
} from './types';

const BASE = '/api';

export async function getPendingValidations(): Promise<PendingValidationDto[]> {
  const res = await fetch(`${BASE}/validations`);
  if (!res.ok) throw new Error(`Failed to fetch validations: ${res.statusText}`);
  return res.json();
}

export async function getPendingCount(): Promise<number> {
  const tracks = await getPendingValidations();
  return tracks.length;
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

export async function getActiveTasksCount(): Promise<number> {
  const tasks = await getTasks();
  return tasks.filter((t) => t.status === 'Pending' || t.status === 'Running').length;
}

// ================================================================================================
// Library — Tracks
// ================================================================================================

export async function getTracks(): Promise<LibraryTrackDto[]> {
  const res = await fetch(`${BASE}/tracks`);
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

export async function getAlbums(): Promise<LibraryAlbumDto[]> {
  const res = await fetch(`${BASE}/albums`);
  if (!res.ok) throw new Error(`Failed to fetch albums: ${res.statusText}`);
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

// ================================================================================================
// Library — Artists
// ================================================================================================

export async function getArtists(): Promise<LibraryArtistDto[]> {
  const res = await fetch(`${BASE}/artists`);
  if (!res.ok) throw new Error(`Failed to fetch artists: ${res.statusText}`);
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

// ================================================================================================
// Sync Schedules
// ================================================================================================

export interface SyncScheduleDto {
  id: number;
  playlist_url: string;
  label: string | null;
  interval_seconds: number;
  enabled: boolean;
  last_run: string | null;
  next_run: string | null;
  created_at: string | null;
}

export async function getSyncSchedules(): Promise<SyncScheduleDto[]> {
  const res = await fetch(`${BASE}/sync-schedules`);
  if (!res.ok) throw new Error(`Failed to fetch sync schedules: ${res.statusText}`);
  return res.json();
}

export async function createSyncSchedule(
  playlist_url: string,
  label: string | null,
  interval_seconds: number,
): Promise<SyncScheduleDto> {
  const res = await fetch(`${BASE}/sync-schedules`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ playlist_url, label: label || null, interval_seconds }),
  });
  if (!res.ok) {
    const err = await res.json().catch(() => ({ message: res.statusText }));
    throw new Error(err.message ?? res.statusText);
  }
  return res.json();
}

export async function updateSyncSchedule(
  id: number,
  patch: { label?: string; interval_seconds?: number; enabled?: boolean },
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

