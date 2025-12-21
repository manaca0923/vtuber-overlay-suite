//! InnerTube API クライアントモジュール
//!
//! YouTubeの内部APIを使用してライブチャットを取得する。
//! 公式API Data v3と異なり、APIキー不要でクォータ制限なし。
//! カスタム絵文字（メンバースタンプ）の画像URLも取得可能。
//!
//! ## 注意事項
//! - 非公式APIのため、仕様変更のリスクあり
//! - Feature Flagで公式APIとの切り替えを推奨
//!
//! ## 実装状態
//! 現在はPoC段階（T13）。本番ポーリングへの統合は将来のタスクで実装予定。
//! リリースビルドでは一部の型・関数がテストコマンドからのみ使用されるため、
//! dead_code警告を抑制している。

// InnerTubeモジュールは現在PoC段階のため、リリースビルドでは
// デバッグ専用のtest_innertube_connectionコマンドからのみ使用される。
// 将来のフル実装に向けて型定義は維持し、警告のみ抑制する。
#![allow(dead_code)]

pub mod client;
pub mod parser;
pub mod types;

pub use client::InnerTubeClient;
pub use parser::{parse_chat_response, clear_emoji_cache};
pub use types::*;
