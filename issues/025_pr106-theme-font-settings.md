# PR#106 テーマ・フォント設定機能レビュー

## 概要

PR#106でテーマカラーとフォント設定を全9ウィジェットに拡張。ローカルテストで発見した問題と対応パターン。

## 発見した問題と対応

### 1. 設定マイグレーションのパターン

**問題**: 既存の設定形式（`theme: 'purple'`）から新形式（`themeSettings: {...}`）への移行時、既存ユーザーの設定が壊れる可能性。

**解決方法**:
```typescript
// OverlaySettings.tsx:79-94
useEffect(() => {
  async function loadAndMigrateSettings() {
    const saved = await loadOverlaySettings();

    // マイグレーション: 旧形式から新形式へ
    if (saved.themeSettings) {
      // 新形式: そのまま使用
      setSettings({ ...DEFAULT_SETTINGS, ...saved, themeSettings: {
        ...DEFAULT_THEME_SETTINGS,
        ...saved.themeSettings
      }});
    } else if (saved.theme || saved.primaryColor) {
      // 旧形式: 変換して使用
      const migratedThemeSettings = {
        ...DEFAULT_THEME_SETTINGS,
        globalTheme: saved.theme || 'white',
        globalPrimaryColor: saved.primaryColor || DEFAULT_THEME_SETTINGS.globalPrimaryColor,
      };
      setSettings({ ...DEFAULT_SETTINGS, ...saved, themeSettings: migratedThemeSettings });
    }
  }
  loadAndMigrateSettings();
}, []);
```

**パターン**:
1. 新フィールドが存在すればそのまま使用（デフォルトとマージ）
2. 旧フィールドが存在すれば変換して使用
3. どちらも存在しなければデフォルト値を使用

### 2. MutationObserverによる動的要素へのスタイル適用

**問題**: コメントがWebSocket経由で動的に追加されるため、CSSカスタムプロパティだけでは色が反映されない場合がある。

**解決方法**:
```javascript
// combined-v2.html:322-348
let commentColorObserver = null;

function setupCommentColorObserver() {
  const container = document.getElementById('comment-container');
  if (!container) return;

  // 既存オブザーバーをクリーンアップ
  if (commentColorObserver) {
    commentColorObserver.disconnect();
  }

  commentColorObserver = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      mutation.addedNodes.forEach((node) => {
        if (node.nodeType === Node.ELEMENT_NODE) {
          applyCommentColor(node);
        }
      });
    });
  });

  commentColorObserver.observe(container, { childList: true, subtree: true });
}

// bfcache復元時のクリーンアップ
window.addEventListener('pageshow', (event) => {
  if (event.persisted) {
    if (commentColorObserver) {
      commentColorObserver.disconnect();
      commentColorObserver = null;
    }
    setupCommentColorObserver();
  }
});
```

**ノウハウ**:
- MutationObserverは`disconnect()`でクリーンアップが必要
- bfcache復元時は再セットアップが必要（issues/010参照）
- 既存オブザーバーがある場合は先にdisconnect

### 3. システムフォント取得（Rust spawn_blocking）

**問題**: `font-kit`のフォント列挙はブロッキング操作で、Tauriのメインスレッドをブロックする。

**解決方法**:
```rust
// src-tauri/src/commands/system.rs
use font_kit::source::SystemSource;

#[tauri::command]
pub async fn get_system_fonts() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(|| {
        let source = SystemSource::new();
        let families = source
            .all_families()
            .map_err(|e| format!("Failed to get fonts: {}", e))?;

        // フィルタリング: 制御文字・異常に長い名前を除外
        const MAX_FONT_NAME_LENGTH: usize = 200;
        let mut fonts: Vec<String> = families
            .into_iter()
            .filter(|name| {
                !name.is_empty()
                    && name.len() <= MAX_FONT_NAME_LENGTH
                    && !name.chars().any(|c| c.is_control())
            })
            .collect();

        fonts.sort();
        fonts.dedup();
        Ok(fonts)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
```

**ノウハウ**:
- ブロッキング操作は`spawn_blocking`でラップ
- フォント名のフィルタリング（セキュリティ対策、issues/002セクション6参照）
- ソート・重複除去でUI表示を最適化

### 4. Google Fonts重複読み込み防止

**問題**: フォント選択のたびにGoogle Fontsが重複ロードされる可能性。

**解決方法**:
```typescript
// FontSelector.tsx / combined-v2.html
const loadedGoogleFonts = new Set<string>();

function loadGoogleFont(fontSpec: string): void {
  if (!fontSpec || loadedGoogleFonts.has(fontSpec)) return;

  // URLバリデーション（issues/002セクション7参照）
  const url = `https://fonts.googleapis.com/css2?family=${fontSpec}&display=swap`;
  try {
    const parsed = new URL(url);
    if (parsed.hostname !== 'fonts.googleapis.com') return;
  } catch {
    return;
  }

  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = url;
  document.head.appendChild(link);
  loadedGoogleFonts.add(fontSpec);
}
```

**ノウハウ**:
- `Set`で既読み込みフォントを管理
- URLホスト名の検証（セキュリティ対策）
- フロントエンド・オーバーレイ両方で同じパターンを適用

### 5. カスタムカラー上限のフロントエンド・バックエンド検証

**問題**: フロントエンドのみで上限チェックしていると、APIリクエスト改ざんで回避される可能性。

**解決方法**:

**フロントエンド側**:
```typescript
// ThemeSelector.tsx:63-78
const MAX_CUSTOM_COLORS = 3;

const handleAddCustomColor = () => {
  if (themeSettings.customColors.length >= MAX_CUSTOM_COLORS) return;
  // 追加処理...
};
```

**バックエンド側（Rust）**:
```rust
// overlay.rs - validate_overlay_settings()
const MAX_CUSTOM_COLORS: usize = 3;

if let Some(ref theme) = settings.theme_settings {
    if theme.custom_colors.len() > MAX_CUSTOM_COLORS {
        return Err(format!(
            "Too many custom colors: {}. Maximum is {}.",
            theme.custom_colors.len(),
            MAX_CUSTOM_COLORS
        ));
    }
}
```

**ノウハウ**:
- 深層防御（issues/002セクション6参照）
- 定数はフロントエンド・バックエンド両方で定義
- バックエンドで最終的な検証を行う

## チェックリスト（テーマ・設定機能実装時）

- [ ] マイグレーション処理を実装したか（旧形式→新形式）
- [ ] デフォルト値とのマージ処理を入れたか
- [ ] MutationObserverのクリーンアップを実装したか
- [ ] bfcache復元時の再セットアップを実装したか
- [ ] ブロッキング操作は`spawn_blocking`でラップしたか
- [ ] 外部リソース（Google Fonts等）の重複読み込み防止を実装したか
- [ ] 上限値はフロントエンド・バックエンド両方で検証しているか
- [ ] フォント名のサニタイズ/フィルタリングを行ったか

## 関連ノウハウ

- **issues/002**: オーバーレイセキュリティ（深層防御、URLバリデーション）
- **issues/010**: bfcache対応（タイマークリア、状態リセット）
- **issues/013**: 防御的プログラミング（型ガード）
- **issues/020**: マジックナンバーの定数化

## 関連PR

- PR#106: テーマ・フォント設定機能の拡張
