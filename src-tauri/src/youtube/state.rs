use serde::{Deserialize, Serialize};
use std::time::Duration;

/// ポーリング状態を管理する構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingState {
    /// 現在のページトークン（次回リクエスト用）
    pub next_page_token: Option<String>,
    /// 推奨ポーリング間隔（ミリ秒）
    pub polling_interval_millis: u64,
    /// 現在のライブチャットID
    pub live_chat_id: String,
    /// 累積クォータ消費量（推定）
    pub quota_used: u64,
    /// ポーリング実施回数
    pub poll_count: u64,
}

impl PollingState {
    /// 新しいポーリング状態を作成
    pub fn new(live_chat_id: String) -> Self {
        Self {
            next_page_token: None,
            polling_interval_millis: 5000, // デフォルト5秒
            live_chat_id,
            quota_used: 0,
            poll_count: 0,
        }
    }

    /// ポーリング間隔をDurationとして取得
    ///
    /// 最低5秒を保証
    pub fn polling_interval(&self) -> Duration {
        let millis = self.polling_interval_millis.max(5000);
        Duration::from_millis(millis)
    }

    /// 状態を更新（API レスポンス受信後に呼び出す）
    pub fn update(&mut self, next_page_token: Option<String>, polling_interval_millis: u64) {
        self.next_page_token = next_page_token;
        self.polling_interval_millis = polling_interval_millis.max(5000);
        self.poll_count += 1;

        // liveChatMessages.list のクォータコストは約5 units
        self.quota_used += 5;
    }

    /// ページトークンをリセット（エラー時など）
    pub fn reset_page_token(&mut self) {
        self.next_page_token = None;
    }

    /// クォータ使用量をリセット（新しい日になった時など）
    pub fn reset_quota(&mut self) {
        self.quota_used = 0;
    }

    /// 残りクォータを推定（デフォルト10,000 units）
    pub fn estimated_remaining_quota(&self) -> i64 {
        const DAILY_QUOTA: i64 = 10_000;
        DAILY_QUOTA - self.quota_used as i64
    }

    /// あと何回ポーリングできるかを推定
    pub fn estimated_remaining_polls(&self) -> i64 {
        self.estimated_remaining_quota() / 5 // 1回あたり5 units
    }
}

// PollingStateManagerは削除しました
// ChatPoller内で直接Arc<Mutex<Option<PollingState>>>を使用しています

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polling_state_creation() {
        let state = PollingState::new("test-chat-id".to_string());
        assert_eq!(state.live_chat_id, "test-chat-id");
        assert_eq!(state.next_page_token, None);
        assert_eq!(state.polling_interval_millis, 5000);
        assert_eq!(state.quota_used, 0);
    }

    #[test]
    fn test_polling_state_update() {
        let mut state = PollingState::new("test-chat-id".to_string());
        state.update(Some("token123".to_string()), 6000);

        assert_eq!(state.next_page_token, Some("token123".to_string()));
        assert_eq!(state.polling_interval_millis, 6000);
        assert_eq!(state.quota_used, 5);
        assert_eq!(state.poll_count, 1);
    }

    #[test]
    fn test_minimum_polling_interval() {
        let mut state = PollingState::new("test-chat-id".to_string());
        state.update(None, 1000); // 1秒を指定

        // 最低5秒が保証される
        assert_eq!(state.polling_interval(), Duration::from_secs(5));
    }

    #[test]
    fn test_quota_estimation() {
        let mut state = PollingState::new("test-chat-id".to_string());

        // 100回ポーリング
        for _ in 0..100 {
            state.update(None, 5000);
        }

        assert_eq!(state.quota_used, 500); // 100 * 5
        assert_eq!(state.estimated_remaining_quota(), 9500); // 10,000 - 500
        assert_eq!(state.estimated_remaining_polls(), 1900); // 9500 / 5
    }

}
