import type { Song } from './song';

export interface Setlist {
  id: string;
  name: string;
  description: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface SetlistSong {
  id: string;
  position: number;
  song: Song;
  startedAt: string | null;
  endedAt: string | null;
  status: SongStatus;
}

export type SongStatus = 'pending' | 'current' | 'done';

export interface SetlistWithSongs {
  setlist: Setlist;
  songs: SetlistSong[];
  currentIndex: number;
}

export interface CreateSetlistInput {
  name: string;
  description?: string;
  [key: string]: unknown;
}
