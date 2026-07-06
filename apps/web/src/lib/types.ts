export interface ArtistDto {
  id: number | null;
  name: string;
}

export interface AlbumDto {
  id: number | null;
  title: string;
  artists: ArtistDto[];
}

export interface ReferenceDto {
  id: number | null;
  ref_type: string;
  platform: string;
  external_id: string | null;
  external_url: string | null;
}

export interface AddReferenceBody {
  ref_type: string;
  platform: string;
  external_id?: string | null;
  external_url?: string | null;
}

export interface PendingValidationDto {
  id: number;
  title: string;
  artists: ArtistDto[];
  album: AlbumDto | null;
  date: string | null;
  genre: string | null;
  cover: string | null;
  duration: number | null;
  track_number: number | null;
  disc_number: number | null;
  label: string | null;
  file_path: string | null;
  validation_reason: string | null;
  references: ReferenceDto[];
}

export interface PatchValidationBody {
  title?: string;
  artists?: string[];
  album_title?: string;
  genre?: string;
  date?: string;
  track_number?: number;
  disc_number?: number;
  label?: string;
  /** YouTube or YouTube Music URL to download from (required for DRM-protected SoundCloud tracks). */
  provider_url?: string;
}

export interface MatchCandidateDto {
  title: string;
  artists: ArtistDto[];
  album: AlbumDto | null;
  date: string | null;
  genre: string | null;
  cover: string | null;
  duration: number | null;
  track_number: number | null;
  disc_number: number | null;
  label: string | null;
  score: number;
  provider: string;
  references: ReferenceDto[];
}

export type TaskStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled' | 'Cancelling';
export type TaskType = 'SyncPlaylist' | 'SyncArtist' | 'SyncAlbum' | 'DownloadTrack' | 'IngestDir';

export interface TaskTrackErrorDto {
  track: string;
  reason: string;
  provider_url: string | null;
}

export interface TaskTrackValidationDto {
  track: string;
  track_id: number | null;
  reason: string | null;
}

export interface TaskStatsDto {
  downloaded: number;
  to_validate: number;
  skipped: number;
  errors: TaskTrackErrorDto[];
  to_validate_tracks: TaskTrackValidationDto[];
  ai_curation: AiCurationProgressDto | null;
}

export interface AiCurationProgressDto {
  processed: number;
  total: number;
}

export interface TaskDto {
  id: number;
  task_type: TaskType;
  status: TaskStatus;
  label: string | null;
  progress: number;
  total: number | null;
  error: string | null;
  stats: TaskStatsDto | null;
  created_at: string | null;
  updated_at: string | null;
}

// ================================================================================================
// Library
// ================================================================================================

export interface LibraryTrackDto {
  id: number;
  title: string;
  artists: { id: number | null; name: string }[];
  album: { id: number | null; title: string } | null;
  date: string | null;
  genre: string | null;
  cover: string | null;
  duration: number | null;
  track_number: number | null;
  disc_number: number | null;
  label: string | null;
  file_path: string | null;
  needs_validation: boolean;
  references: ReferenceDto[];
}

export interface UpdateTrackBody {
  title?: string;
  artists?: string[];
  album_title?: string;
  genre?: string;
  date?: string;
  track_number?: number;
  disc_number?: number;
  label?: string;
  cover?: string;
}

export interface LibraryAlbumDto {
  id: number;
  title: string;
  artists: { id: number | null; name: string }[];
  album_type: string;
  cover: string | null;
  date: string | null;
  references: ReferenceDto[];
}

export interface UpdateAlbumBody {
  title?: string;
  date?: string;
  cover?: string;
}

export interface LibraryArtistDto {
  id: number;
  name: string;
  icon: string | null;
  references: ReferenceDto[];
}

export interface UpdateArtistBody {
  name?: string;
  icon?: string;
}

export interface LibraryPlaylistDto {
  id: number;
  name: string;
  source: string;
  source_url: string | null;
  cover: string | null;
}

export interface PlaylistTrackDto {
  id: number;
  title: string;
  artists: { id: number | null; name: string }[];
  album: { id: number | null; title: string } | null;
  duration: number | null;
  cover: string | null;
  genre: string | null;
}
