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

export type TaskStatus = 'Pending' | 'Running' | 'Completed' | 'Failed';
export type TaskType = 'SyncPlaylist' | 'DownloadTrack';

export interface TaskDto {
  id: number;
  task_type: TaskType;
  status: TaskStatus;
  label: string | null;
  progress: number;
  total: number | null;
  error: string | null;
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
}

export interface UpdateArtistBody {
  name?: string;
  icon?: string;
}
