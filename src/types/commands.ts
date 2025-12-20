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
  invoke<void>('add_song_to_setlist', { setlist_id: setlistId, song_id: songId });

export const removeSongFromSetlist = (setlistId: string, setlistSongId: string) =>
  invoke<void>('remove_song_from_setlist', { setlist_id: setlistId, setlist_song_id: setlistSongId });

// Song control commands
export const setCurrentSong = (setlistId: string, position: number) =>
  invoke<void>('set_current_song', { setlist_id: setlistId, position });

export const nextSong = (setlistId: string) =>
  invoke<void>('next_song', { setlist_id: setlistId });

export const previousSong = (setlistId: string) =>
  invoke<void>('previous_song', { setlist_id: setlistId });

export const reorderSetlistSongs = (setlistId: string, setlistSongIds: string[]) =>
  invoke<void>('reorder_setlist_songs', { setlist_id: setlistId, setlist_song_ids: setlistSongIds });

export const broadcastSetlistUpdate = (setlistId: string) =>
  invoke<void>('broadcast_setlist_update', { setlist_id: setlistId });

// Test mode commands
export const sendTestComment = (commentText: string, authorName: string) =>
  invoke<void>('send_test_comment', { comment_text: commentText, author_name: authorName });
