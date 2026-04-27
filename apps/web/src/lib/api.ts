import type { PendingValidationDto, PatchValidationBody } from './types';

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

export type DownloadResultTrack = {
  type: 'track';
  title: string;
  artists: string[];
  needs_validation: boolean;
};

export type DownloadResultPlaylist = {
  type: 'playlist';
  downloaded: number;
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
