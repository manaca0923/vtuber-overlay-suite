/**
 * ウィザード設定型
 *
 * Rust側でrename_allなしのためsnake_caseでフィールド名を定義
 * @see src-tauri/src/commands/youtube.rs WizardSettingsData
 */
export interface WizardSettingsData {
  video_id: string;
  live_chat_id: string;
  /** 保存日時（ISO 8601形式） */
  saved_at: string;
  /** 同梱APIキー使用フラグ（後方互換性のためオプショナル） */
  use_bundled_key?: boolean;
}
