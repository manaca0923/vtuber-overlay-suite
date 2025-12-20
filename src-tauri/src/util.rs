/// APIキーをマスキングしてログ出力用の文字列を生成
///
/// APIキーの最初の4文字と最後の4文字のみを表示し、中間を***でマスキング
///
/// # Examples
/// ```
/// use app_lib::util::mask_api_key;
/// let masked = mask_api_key("AIzaSyABC123def456GHI789");
/// assert_eq!(masked, "AIza***I789");
/// ```
pub fn mask_api_key(api_key: &str) -> String {
    if api_key.is_empty() {
        return "***".to_string();
    }

    // 文字数（バイト数ではなく）で判定
    let char_count = api_key.chars().count();
    if char_count <= 8 {
        // 短いキーは全体をマスク
        return "***".to_string();
    }

    // UTF-8安全な文字単位での分割
    let chars: Vec<char> = api_key.chars().collect();
    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars.iter().skip(char_count - 4).collect();
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

        // 非ASCII文字（マルチバイト文字）- 9文字
        assert_eq!(mask_api_key("こんにちは世界です"), "こんにち***世界です");

        // 絵文字 - 9文字（注: 一部の絵文字は複数のコードポイントを持つ可能性あり）
        assert_eq!(mask_api_key("🔑🔐🔓🔒🔏🔎🔍🔐🔑"), "🔑🔐🔓🔒***🔎🔍🔐🔑");

        // 混在（ASCII + 日本語）- 10文字
        assert_eq!(mask_api_key("APIキー12345"), "APIキ***2345");
    }
}
