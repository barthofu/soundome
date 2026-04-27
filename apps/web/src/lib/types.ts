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
