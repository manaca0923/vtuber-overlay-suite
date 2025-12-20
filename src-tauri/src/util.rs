/// APIキーをマスキングしてログ出力用の文字列を生成
///
/// APIキーの最初の4文字と最後の4文字のみを表示し、中間を***でマスキング
///
/// # Examples
/// ```
/// let masked = mask_api_key("AIzaSyABC123def456GHI789");
/// assert_eq!(masked, "AIza***I789");
/// ```
pub fn mask_api_key(api_key: &str) -> String {
    if api_key.is_empty() {
        return "***".to_string();
    }

    let len = api_key.len();
    if len <= 8 {
        // 短いキーは全体をマスク
        return "***".to_string();
    }

    let prefix = &api_key[..4];
    let suffix = &api_key[len - 4..];
    format!("{}***{}", prefix, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        // 通常のAPIキー
        assert_eq!(
            mask_api_key("AIzaSyABC123def456GHI789"),
            "AIza***I789"
        );

        // 短いキー
        assert_eq!(mask_api_key("short"), "***");

        // 空文字列
        assert_eq!(mask_api_key(""), "***");

        // 8文字ちょうど
        assert_eq!(mask_api_key("12345678"), "***");

        // 9文字（マスキング開始）
        assert_eq!(mask_api_key("123456789"), "1234***6789");
    }
}
