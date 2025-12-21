//! InnerTube API クライアントモジュール
//!
//! YouTubeの内部APIを使用してライブチャットを取得する。
//! 公式API Data v3と異なり、APIキー不要でクォータ制限なし。
//! カスタム絵文字（メンバースタンプ）の画像URLも取得可能。
//!
//! ## 注意事項
//! - 非公式APIのため、仕様変更のリスクあり
//! - Feature Flagで公式APIとの切り替えを推奨

pub mod client;
pub mod parser;
pub mod types;

pub use client::InnerTubeClient;
pub use parser::parse_chat_response;
pub use types::*;
