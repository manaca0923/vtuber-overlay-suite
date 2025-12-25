export interface Song {
  id: string;
  title: string;
  artist: string | null;
  category: string | null;
  tags: string | null; // JSON string from Rust
  durationSeconds: number | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateSongInput {
  title: string;
  artist?: string | null;
  category?: string | null;
  tags?: string[] | null;
  duration_seconds?: number | null;
  [key: string]: unknown;
}

export interface UpdateSongInput extends CreateSongInput {
  id: string;
}

/**
 * Parse tags from JSON string to array
 */
export function parseTags(song: Song): string[] {
  if (!song.tags) return [];
  try {
    return JSON.parse(song.tags) as string[];
  } catch {
    return [];
  }
}
