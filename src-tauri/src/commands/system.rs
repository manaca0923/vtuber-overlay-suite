use font_kit::source::SystemSource;

/// フォント名の最大長（セキュリティ対策）
const MAX_FONT_NAME_LENGTH: usize = 200;

/// システムにインストールされているフォント一覧を取得
///
/// # Returns
/// フォントファミリー名のリスト（アルファベット順）
///
/// # Security
/// - 制御文字を含むフォント名を除外
/// - 異常に長いフォント名を除外（MAX_FONT_NAME_LENGTH文字以上）
/// - 深層防御: フロントエンド側でもsanitizeFontFamily()でサニタイズ
#[tauri::command]
pub async fn get_system_fonts() -> Result<Vec<String>, String> {
    // font-kitはブロッキング操作なので、spawn_blockingで実行
    tokio::task::spawn_blocking(|| {
        let source = SystemSource::new();
        let families = source
            .all_families()
            .map_err(|e| format!("Failed to get fonts: {}", e))?;

        // フィルタリング: 制御文字や異常に長い名前を除外（セキュリティ対策）
        let mut fonts: Vec<String> = families
            .into_iter()
            .filter(|name| {
                // 空文字や異常に長い名前を除外
                !name.is_empty()
                    && name.len() <= MAX_FONT_NAME_LENGTH
                    // 制御文字を含むフォント名を除外
                    && !name.chars().any(|c| c.is_control())
            })
            .collect();

        // アルファベット順でソート
        fonts.sort();

        Ok(fonts)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
