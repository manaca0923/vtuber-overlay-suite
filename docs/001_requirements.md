# VTuber配信支援ツール - 要件サマリー

> 原本: `VTuber配信支援ツール_要件定義書_v1.2.0.md`

## プロダクト概要

**コンセプト**: 「わんコメ＋セトリスタ」の上位互換オールインワンツール

### コアバリュー（MVP）
| 価値 | 説明 |
|------|------|
| 統合 | コメント表示＋セットリスト＋画面演出を単一UIに |
| 神デザイン | プロクオリティのテンプレート（ミナトデザイン・唐揚丸レベル） |
| 安定動作 | 2時間配信でも落ちない |

---

## MVP機能一覧（Phase 1）

| ID | 機能名 | 概要 | 詳細ドキュメント |
|----|--------|------|------------------|
| F01 | コメント取得・表示 | YouTube Liveからコメント取得、OBS表示 | [200_youtube-api.md](./200_youtube-api.md) |
| F02 | セットリスト管理 | 楽曲リスト作成、曲順管理、OBS表示 | [400_data-models.md](./400_data-models.md) |
| F03 | 高品質テンプレート | プロデザイナー制作の統一デザインセット | [300_overlay-specs.md](./300_overlay-specs.md) |
| F04 | OBSブラウザソース連携 | URLコピーでOBSに追加 | [300_overlay-specs.md](./300_overlay-specs.md) |

### Phase 2以降
- F05: コメント読み上げ（VOICEVOX連携）
- F06: VTube Studio連携
- F07: 物理演出エンジン

---

## 受け入れ基準（MVP）

### F01: コメント取得・表示
- [ ] 2時間配信で取得停止しない（自動復帰含む）
- [ ] 遅延が体感許容内
- [ ] gRPC Streaming（公式API）で安定動作（メイン経路）
- [ ] InnerTube APIで認証不要のバックアップ動作
- [ ] YouTube Data API v3 ポーリングで互換動作（BYOK）

### F02: セットリスト管理
- [ ] ドラッグ&ドロップで曲順編集
- [ ] タイムスタンプ自動記録
- [ ] YouTube概要欄用コピー出力

### F03: 高品質テンプレート
- [ ] VTuberモニター5名中4名以上が「使いたい」と回答
- [ ] 簡易設定（色・位置・ON/OFF）が可能

### F04: OBS連携
- [ ] OBSで簡単に動く（URLコピー→貼り付け）
- [ ] 配信中も安定（落ちない・ズレない）

### 全体
- [ ] 起動時更新チェック→通知が動作
- [ ] 2時間配信中のクラッシュ率 < 1%

---

## 対象環境

| 項目 | 仕様 |
|------|------|
| OS | Windows（優先）/ macOS |
| 配信 | YouTube Live |
| 連携 | OBS Studio |
| デスクトップ | Tauri 2.0 |

---

## 制約・前提

1. **gRPC優先**: gRPC Streaming（公式API）をメインで使用（安定性・低クォータ消費）
2. **InnerTubeバックアップ**: 認証不要のInnerTube APIはバックアップとして利用可能
3. **REST API互換**: YouTube Data API v3 ポーリングも利用可能（BYOK・互換モード）
4. **OAuth回避（MVP）**: API key中心、OAuthはPhase 2
5. **品質優先**: 「多いけど使われない」ではなく少数精鋭

> **Note**: InnerTubeは非公式APIのため、YouTube側の仕様変更で動作しなくなる可能性があります。gRPC/REST APIをメインでご利用ください。

---

## 関連ドキュメント

- [100_architecture.md](./100_architecture.md) - 技術アーキテクチャ
- [200_youtube-api.md](./200_youtube-api.md) - YouTube API仕様
- [300_overlay-specs.md](./300_overlay-specs.md) - オーバーレイ仕様
- [400_data-models.md](./400_data-models.md) - データモデル
- [900_tasks.md](./900_tasks.md) - タスク分解・進捗
