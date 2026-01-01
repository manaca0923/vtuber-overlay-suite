/**
 * ウィザード設定型
 *
 * Rust側でrename_allなしのためsnake_caseでフィールド名を定義
 * @see src-tauri/src/commands/youtube.rs WizardSettingsData
 */
export interface WizardSettingsData {
  video_id: string;
  live_chat_id: string;
  saved_at?: string;
  use_bundled_key?: boolean;
}
