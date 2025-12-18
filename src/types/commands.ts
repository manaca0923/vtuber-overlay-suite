import { invoke } from '@tauri-apps/api/core';
import type { Song, CreateSongInput, UpdateSongInput } from './song';
import type { Setlist, SetlistWithSongs, CreateSetlistInput } from './setlist';

// Song commands
export const getSongs = () => invoke<Song[]>('get_songs');

export const createSong = (input: CreateSongInput) =>
  invoke<Song>('create_song', input);

export const updateSong = (input: UpdateSongInput) =>
  invoke<Song>('update_song', input);

export const deleteSong = (id: string) =>
  invoke<void>('delete_song', { id });

// Setlist commands
export const getSetlists = () => invoke<Setlist[]>('get_setlists');

export const getSetlistWithSongs = (id: string) =>
  invoke<SetlistWithSongs>('get_setlist_with_songs', { id });

export const createSetlist = (input: CreateSetlistInput) =>
  invoke<Setlist>('create_setlist', input);

export const deleteSetlist = (id: string) =>
  invoke<void>('delete_setlist', { id });

export const addSongToSetlist = (setlistId: string, songId: string) =>
  invoke<void>('add_song_to_setlist', { setlistId, songId });

export const removeSongFromSetlist = (setlistId: string, setlistSongId: string) =>
  invoke<void>('remove_song_from_setlist', { setlistId, setlistSongId });

// Song control commands
export const setCurrentSong = (setlistId: string, position: number) =>
  invoke<void>('set_current_song', { setlistId, position });

export const nextSong = (setlistId: string) =>
  invoke<void>('next_song', { setlistId });

export const previousSong = (setlistId: string) =>
  invoke<void>('previous_song', { setlistId });

export const reorderSetlistSongs = (setlistId: string, setlistSongIds: string[]) =>
  invoke<void>('reorder_setlist_songs', { setlistId, setlistSongIds });
