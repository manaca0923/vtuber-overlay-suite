use font_kit::source::SystemSource;

/// システムにインストールされているフォント一覧を取得
///
/// # Returns
/// フォントファミリー名のリスト（アルファベット順）
#[tauri::command]
pub async fn get_system_fonts() -> Result<Vec<String>, String> {
    // font-kitはブロッキング操作なので、spawn_blockingで実行
    tokio::task::spawn_blocking(|| {
        let source = SystemSource::new();
        let families = source
            .all_families()
            .map_err(|e| format!("Failed to get fonts: {}", e))?;

        // アルファベット順でソート
        let mut fonts: Vec<String> = families.into_iter().collect();
        fonts.sort();

        Ok(fonts)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
