//! テンプレート型定義
//!
//! 3カラムレイアウト v2 のテンプレート設定
//!
//! 参照: docs/3カラム_要件仕様書_追記版_v1.1.md
//! Schema: src-tauri/schemas/template-mvp-1.0.json

use serde::{Deserialize, Serialize};

use super::types::SlotId;

// ===== コンポーネントタイプ =====

/// コンポーネント種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentType {
    ClockWidget,
    WeatherWidget,
    ChatLog,
    SuperChatCard,
    BrandBlock,
    MainAvatarStage,
    ChannelBadge,
    SetList,
    #[serde(rename = "KPIBlock")]
    KpiBlock,
    PromoPanel,
    QueueList,
}

// ===== レイアウト設定 =====

/// 3カラムレイアウト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateLayout {
    /// レイアウトタイプ（"threeColumn"固定）
    #[serde(rename = "type")]
    pub layout_type: String,
    /// 左カラム比率（0.18〜0.28）
    pub left_pct: f64,
    /// 中央カラム比率（0.44〜0.64）
    pub center_pct: f64,
    /// 右カラム比率（0.18〜0.28）
    pub right_pct: f64,
    /// カラム間ガター（0〜64px）
    pub gutter_px: u32,
}

impl Default for TemplateLayout {
    fn default() -> Self {
        Self {
            layout_type: "threeColumn".to_string(),
            left_pct: 0.22,
            center_pct: 0.56,
            right_pct: 0.22,
            gutter_px: 24,
        }
    }
}

// ===== セーフエリア設定 =====

/// セーフエリア設定（各辺0.0〜0.10）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSafeArea {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Default for TemplateSafeArea {
    fn default() -> Self {
        Self {
            top: 0.04,
            right: 0.04,
            bottom: 0.05,
            left: 0.04,
        }
    }
}

// ===== テーマ設定 =====

/// パネルスタイル
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplatePanelStyle {
    /// 背景色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bg: Option<String>,
    /// ブラー量（0〜24px）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur_px: Option<u32>,
    /// 角丸（0〜32px）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius_px: Option<u32>,
}

/// 影スタイル
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateShadowStyle {
    /// 有効/無効
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// ブラー量（0〜24）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur: Option<u32>,
    /// 不透明度（0.0〜1.0）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// X方向オフセット（-20〜20）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_x: Option<i32>,
    /// Y方向オフセット（-20〜20）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_y: Option<i32>,
}

/// アウトラインスタイル
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateOutlineStyle {
    /// 有効/無効
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// 線幅（0〜6px）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// 色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// テーマ設定
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateTheme {
    /// フォントファミリー
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    /// テキスト色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    /// パネルスタイル
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panel: Option<TemplatePanelStyle>,
    /// 影スタイル
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<TemplateShadowStyle>,
    /// アウトラインスタイル
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<TemplateOutlineStyle>,
}

// ===== コンポーネント設定 =====

/// コンポーネントルール
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRules {
    /// 最大行数（4〜14）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<u32>,
    /// 最大アイテム数（6〜20）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u32>,
    /// サイクル秒数（10〜120）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle_sec: Option<u32>,
    /// 表示秒数（3〜15）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_sec: Option<u32>,
}

/// コンポーネントチューニング
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentTuning {
    /// X方向オフセット（-40〜40px）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_x: Option<i32>,
    /// Y方向オフセット（-40〜40px）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_y: Option<i32>,
}

/// コンポーネント設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateComponent {
    /// 一意のID
    pub id: String,
    /// コンポーネント種別
    #[serde(rename = "type")]
    pub component_type: ComponentType,
    /// 配置slot
    pub slot: SlotId,
    /// 有効/無効
    pub enabled: bool,
    /// スタイル設定（任意）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<serde_json::Value>,
    /// ルール設定（任意）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<ComponentRules>,
    /// チューニング設定（任意）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuning: Option<ComponentTuning>,
}

// ===== テンプレート全体 =====

/// テンプレート設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    /// レイアウト設定
    pub layout: TemplateLayout,
    /// セーフエリア設定
    pub safe_area_pct: TemplateSafeArea,
    /// テーマ設定（任意）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<TemplateTheme>,
    /// コンポーネント配列
    pub components: Vec<TemplateComponent>,
}

/// デフォルトテンプレート
///
/// 注: componentsは空のVecで初期化。
/// これはJSON Schema（minItems: 1）に違反しますが、
/// テンプレート作成UIの初期状態として使用するため意図的です。
/// 実際のテンプレート保存時はcomponentsが追加されることを想定しています。
impl Default for Template {
    fn default() -> Self {
        Self {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: Vec::new(),
        }
    }
}

// ===== クランプユーティリティ =====

/// クランプ関数群
pub mod clamp {
    /// tuning.offsetX/Y をクランプ（-40〜40）
    pub fn offset(value: i32) -> i32 {
        value.clamp(-40, 40)
    }

    /// rules.maxLines をクランプ（4〜14）
    pub fn max_lines(value: u32) -> u32 {
        value.clamp(4, 14)
    }

    /// rules.maxItems をクランプ（3〜20）
    /// SetList推奨: 14 (範囲6〜20)、QueueList推奨: 6 (範囲3〜10)
    pub fn max_items(value: u32) -> u32 {
        value.clamp(3, 20)
    }

    /// rules.cycleSec をクランプ（10〜120）
    pub fn cycle_sec(value: u32) -> u32 {
        value.clamp(10, 120)
    }

    /// rules.showSec をクランプ（3〜15）
    pub fn show_sec(value: u32) -> u32 {
        value.clamp(3, 15)
    }

    /// layout.leftPct をクランプ（0.18〜0.28）
    pub fn left_pct(value: f64) -> f64 {
        value.clamp(0.18, 0.28)
    }

    /// layout.centerPct をクランプ（0.44〜0.64）
    pub fn center_pct(value: f64) -> f64 {
        value.clamp(0.44, 0.64)
    }

    /// layout.rightPct をクランプ（0.18〜0.28）
    pub fn right_pct(value: f64) -> f64 {
        value.clamp(0.18, 0.28)
    }

    /// layout.gutterPx をクランプ（0〜64）
    pub fn gutter_px(value: u32) -> u32 {
        value.clamp(0, 64)
    }

    /// safeArea をクランプ（0.0〜0.10）
    pub fn safe_area(value: f64) -> f64 {
        value.clamp(0.0, 0.10)
    }

    /// theme.panel.blurPx をクランプ（0〜24）
    pub fn blur_px(value: u32) -> u32 {
        value.clamp(0, 24)
    }

    /// theme.panel.radiusPx をクランプ（0〜32）
    pub fn radius_px(value: u32) -> u32 {
        value.clamp(0, 32)
    }

    /// theme.shadow.blur をクランプ（0〜24）
    pub fn shadow_blur(value: u32) -> u32 {
        value.clamp(0, 24)
    }

    /// theme.shadow.opacity をクランプ（0.0〜1.0）
    pub fn shadow_opacity(value: f64) -> f64 {
        value.clamp(0.0, 1.0)
    }

    /// theme.shadow.offset をクランプ（-20〜20）
    pub fn shadow_offset(value: i32) -> i32 {
        value.clamp(-20, 20)
    }

    /// theme.outline.width をクランプ（0〜6）
    pub fn outline_width(value: u32) -> u32 {
        value.clamp(0, 6)
    }
}

// ===== バリデーション =====

impl Template {
    /// テンプレートをバリデーション＆クランプ
    /// 不正な値はクランプして適用
    pub fn validate_and_clamp(&mut self) {
        // layout_typeを強制（現在はthreeColumnのみサポート）
        self.layout.layout_type = "threeColumn".to_string();

        // layout クランプ
        self.layout.left_pct = clamp::left_pct(self.layout.left_pct);
        self.layout.center_pct = clamp::center_pct(self.layout.center_pct);
        self.layout.right_pct = clamp::right_pct(self.layout.right_pct);
        self.layout.gutter_px = clamp::gutter_px(self.layout.gutter_px);

        // safeAreaPct クランプ
        self.safe_area_pct.top = clamp::safe_area(self.safe_area_pct.top);
        self.safe_area_pct.right = clamp::safe_area(self.safe_area_pct.right);
        self.safe_area_pct.bottom = clamp::safe_area(self.safe_area_pct.bottom);
        self.safe_area_pct.left = clamp::safe_area(self.safe_area_pct.left);

        // theme クランプ
        if let Some(ref mut theme) = self.theme {
            if let Some(ref mut panel) = theme.panel {
                if let Some(blur) = panel.blur_px {
                    panel.blur_px = Some(clamp::blur_px(blur));
                }
                if let Some(radius) = panel.radius_px {
                    panel.radius_px = Some(clamp::radius_px(radius));
                }
            }
            if let Some(ref mut shadow) = theme.shadow {
                if let Some(blur) = shadow.blur {
                    shadow.blur = Some(clamp::shadow_blur(blur));
                }
                if let Some(opacity) = shadow.opacity {
                    shadow.opacity = Some(clamp::shadow_opacity(opacity));
                }
                if let Some(offset_x) = shadow.offset_x {
                    shadow.offset_x = Some(clamp::shadow_offset(offset_x));
                }
                if let Some(offset_y) = shadow.offset_y {
                    shadow.offset_y = Some(clamp::shadow_offset(offset_y));
                }
            }
            if let Some(ref mut outline) = theme.outline {
                if let Some(width) = outline.width {
                    outline.width = Some(clamp::outline_width(width));
                }
            }
        }

        // components クランプ
        for comp in &mut self.components {
            if let Some(ref mut rules) = comp.rules {
                if let Some(max_lines) = rules.max_lines {
                    rules.max_lines = Some(clamp::max_lines(max_lines));
                }
                if let Some(max_items) = rules.max_items {
                    rules.max_items = Some(clamp::max_items(max_items));
                }
                if let Some(cycle_sec) = rules.cycle_sec {
                    rules.cycle_sec = Some(clamp::cycle_sec(cycle_sec));
                }
                if let Some(show_sec) = rules.show_sec {
                    rules.show_sec = Some(clamp::show_sec(show_sec));
                }
            }
            if let Some(ref mut tuning) = comp.tuning {
                if let Some(offset_x) = tuning.offset_x {
                    tuning.offset_x = Some(clamp::offset(offset_x));
                }
                if let Some(offset_y) = tuning.offset_y {
                    tuning.offset_y = Some(clamp::offset(offset_y));
                }
            }
        }
    }

    /// 有効なコンポーネントでslotの重複がないかチェック
    /// （enabled=trueのコンポーネントのみ対象）
    pub fn has_slot_duplicates(&self) -> bool {
        use std::collections::HashSet;
        let mut used_slots = HashSet::new();
        for comp in &self.components {
            if comp.enabled {
                if used_slots.contains(&comp.slot) {
                    return true;
                }
                used_slots.insert(comp.slot);
            }
        }
        false
    }

    /// コンポーネントIDの重複がないかチェック
    pub fn has_id_duplicates(&self) -> bool {
        use std::collections::HashSet;
        let mut used_ids = HashSet::new();
        for comp in &self.components {
            if used_ids.contains(&comp.id) {
                return true;
            }
            used_ids.insert(&comp.id);
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_offset() {
        assert_eq!(clamp::offset(-50), -40);
        assert_eq!(clamp::offset(50), 40);
        assert_eq!(clamp::offset(0), 0);
        assert_eq!(clamp::offset(-40), -40);
        assert_eq!(clamp::offset(40), 40);
    }

    #[test]
    fn test_clamp_max_lines() {
        assert_eq!(clamp::max_lines(1), 4);
        assert_eq!(clamp::max_lines(20), 14);
        assert_eq!(clamp::max_lines(10), 10);
    }

    #[test]
    fn test_clamp_layout_pct() {
        assert_eq!(clamp::left_pct(0.1), 0.18);
        assert_eq!(clamp::left_pct(0.5), 0.28);
        assert_eq!(clamp::left_pct(0.22), 0.22);

        assert_eq!(clamp::center_pct(0.3), 0.44);
        assert_eq!(clamp::center_pct(0.8), 0.64);
        assert_eq!(clamp::center_pct(0.56), 0.56);
    }

    #[test]
    fn test_default_template() {
        let template = Template::default();
        assert_eq!(template.layout.layout_type, "threeColumn");
        assert_eq!(template.layout.left_pct, 0.22);
        assert_eq!(template.layout.center_pct, 0.56);
        assert_eq!(template.layout.right_pct, 0.22);
        assert_eq!(template.layout.gutter_px, 24);
    }

    #[test]
    fn test_validate_and_clamp() {
        let mut template = Template {
            layout: TemplateLayout {
                layout_type: "threeColumn".to_string(),
                left_pct: 0.1,   // should clamp to 0.18
                center_pct: 0.8, // should clamp to 0.64
                right_pct: 0.5,  // should clamp to 0.28
                gutter_px: 100,  // should clamp to 64
            },
            safe_area_pct: TemplateSafeArea {
                top: 0.2,    // should clamp to 0.10
                right: 0.04,
                bottom: 0.05,
                left: -0.1, // should clamp to 0.0
            },
            theme: None,
            components: vec![TemplateComponent {
                id: "test".to_string(),
                component_type: ComponentType::ChatLog,
                slot: SlotId::LeftMiddle,
                enabled: true,
                style: None,
                rules: Some(ComponentRules {
                    max_lines: Some(100), // should clamp to 14
                    max_items: None,
                    cycle_sec: None,
                    show_sec: None,
                }),
                tuning: Some(ComponentTuning {
                    offset_x: Some(-100), // should clamp to -40
                    offset_y: Some(100),  // should clamp to 40
                }),
            }],
        };

        template.validate_and_clamp();

        assert_eq!(template.layout.left_pct, 0.18);
        assert_eq!(template.layout.center_pct, 0.64);
        assert_eq!(template.layout.right_pct, 0.28);
        assert_eq!(template.layout.gutter_px, 64);
        assert_eq!(template.safe_area_pct.top, 0.10);
        assert_eq!(template.safe_area_pct.left, 0.0);

        let comp = &template.components[0];
        assert_eq!(comp.rules.as_ref().unwrap().max_lines, Some(14));
        assert_eq!(comp.tuning.as_ref().unwrap().offset_x, Some(-40));
        assert_eq!(comp.tuning.as_ref().unwrap().offset_y, Some(40));
    }

    #[test]
    fn test_slot_duplicates() {
        let template = Template {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: vec![
                TemplateComponent {
                    id: "comp1".to_string(),
                    component_type: ComponentType::ChatLog,
                    slot: SlotId::LeftMiddle,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
                TemplateComponent {
                    id: "comp2".to_string(),
                    component_type: ComponentType::SetList,
                    slot: SlotId::LeftMiddle, // duplicate!
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
            ],
        };

        assert!(template.has_slot_duplicates());
    }

    #[test]
    fn test_no_slot_duplicates_when_disabled() {
        let template = Template {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: vec![
                TemplateComponent {
                    id: "comp1".to_string(),
                    component_type: ComponentType::ChatLog,
                    slot: SlotId::LeftMiddle,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
                TemplateComponent {
                    id: "comp2".to_string(),
                    component_type: ComponentType::SetList,
                    slot: SlotId::LeftMiddle, // same slot but disabled
                    enabled: false,
                    style: None,
                    rules: None,
                    tuning: None,
                },
            ],
        };

        assert!(!template.has_slot_duplicates());
    }

    #[test]
    fn test_id_duplicates() {
        let template = Template {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: vec![
                TemplateComponent {
                    id: "same-id".to_string(), // duplicate!
                    component_type: ComponentType::ChatLog,
                    slot: SlotId::LeftMiddle,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
                TemplateComponent {
                    id: "same-id".to_string(), // duplicate!
                    component_type: ComponentType::SetList,
                    slot: SlotId::RightUpper,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
            ],
        };

        assert!(template.has_id_duplicates());
    }

    #[test]
    fn test_no_id_duplicates() {
        let template = Template {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: vec![
                TemplateComponent {
                    id: "comp1".to_string(),
                    component_type: ComponentType::ChatLog,
                    slot: SlotId::LeftMiddle,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
                TemplateComponent {
                    id: "comp2".to_string(),
                    component_type: ComponentType::SetList,
                    slot: SlotId::RightUpper,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
            ],
        };

        assert!(!template.has_id_duplicates());
    }

    #[test]
    fn test_layout_type_forced_to_three_column() {
        let mut template = Template {
            layout: TemplateLayout {
                layout_type: "invalid".to_string(),
                left_pct: 0.22,
                center_pct: 0.56,
                right_pct: 0.22,
                gutter_px: 24,
            },
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: Vec::new(),
        };

        template.validate_and_clamp();
        assert_eq!(template.layout.layout_type, "threeColumn");
    }
}
