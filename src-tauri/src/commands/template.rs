//! テンプレート関連コマンド
//!
//! テンプレート設定のバリデーション・保存・読み込み

use crate::server::template_types::Template;

/// テンプレートをバリデーション＆クランプ
///
/// 不正な値はクランプして適用し、検証済みのテンプレートを返す
#[tauri::command]
pub fn validate_template(mut template: Template) -> Result<Template, String> {
    // バリデーション＆クランプ
    template.validate_and_clamp();

    // slot重複チェック
    if template.has_slot_duplicates() {
        return Err("有効なコンポーネントでslotが重複しています".to_string());
    }

    // コンポーネントID重複チェック
    if template.has_id_duplicates() {
        return Err("コンポーネントIDが重複しています".to_string());
    }

    // コンポーネントが少なくとも1つあるかチェック
    if template.components.is_empty() {
        return Err("コンポーネントが1つも定義されていません".to_string());
    }

    // layoutの合計チェック（左+中央+右が1.0に近いかどうか）
    let total = template.layout.left_pct + template.layout.center_pct + template.layout.right_pct;
    if total < 0.95 || total > 1.05 {
        log::warn!(
            "レイアウト比率の合計が1.0から離れています: {} (left={}, center={}, right={})",
            total,
            template.layout.left_pct,
            template.layout.center_pct,
            template.layout.right_pct
        );
    }

    Ok(template)
}

/// テンプレートのデフォルト設定を取得
#[tauri::command]
pub fn get_default_template() -> Template {
    Template::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::template_types::{
        ComponentRules, ComponentTuning, ComponentType, LayoutType, TemplateComponent,
        TemplateLayout, TemplateSafeArea,
    };
    use crate::server::types::SlotId;

    fn create_test_template() -> Template {
        Template {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: vec![TemplateComponent {
                id: "test-chatlog".to_string(),
                component_type: ComponentType::ChatLog,
                slot: SlotId::LeftMiddle,
                enabled: true,
                style: None,
                rules: Some(ComponentRules {
                    max_lines: Some(10),
                    max_items: None,
                    cycle_sec: None,
                    show_sec: None,
                }),
                tuning: None,
            }],
        }
    }

    #[test]
    fn test_validate_template_success() {
        let template = create_test_template();
        let result = validate_template(template);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_template_clamps_values() {
        let template = Template {
            layout: TemplateLayout {
                layout_type: LayoutType::ThreeColumn,
                left_pct: 0.1,   // should clamp to 0.18
                center_pct: 0.8, // should clamp to 0.64
                right_pct: 0.1,  // should clamp to 0.18
                gutter_px: 100,  // should clamp to 64
            },
            safe_area_pct: TemplateSafeArea::default(),
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

        let result = validate_template(template).unwrap();

        assert_eq!(result.layout.left_pct, 0.18);
        assert_eq!(result.layout.center_pct, 0.64);
        assert_eq!(result.layout.right_pct, 0.18);
        assert_eq!(result.layout.gutter_px, 64);

        let comp = &result.components[0];
        assert_eq!(comp.rules.as_ref().unwrap().max_lines, Some(14));
        assert_eq!(comp.tuning.as_ref().unwrap().offset_x, Some(-40));
        assert_eq!(comp.tuning.as_ref().unwrap().offset_y, Some(40));
    }

    #[test]
    fn test_validate_template_rejects_empty_components() {
        let template = Template {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: vec![],
        };

        let result = validate_template(template);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("コンポーネントが1つも定義されていません"));
    }

    #[test]
    fn test_validate_template_rejects_slot_duplicates() {
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

        let result = validate_template(template);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("slotが重複"));
    }

    #[test]
    fn test_validate_template_rejects_id_duplicates() {
        let template = Template {
            layout: TemplateLayout::default(),
            safe_area_pct: TemplateSafeArea::default(),
            theme: None,
            components: vec![
                TemplateComponent {
                    id: "same-id".to_string(), // duplicate ID!
                    component_type: ComponentType::ChatLog,
                    slot: SlotId::LeftMiddle,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
                TemplateComponent {
                    id: "same-id".to_string(), // duplicate ID!
                    component_type: ComponentType::SetList,
                    slot: SlotId::RightUpper,
                    enabled: true,
                    style: None,
                    rules: None,
                    tuning: None,
                },
            ],
        };

        let result = validate_template(template);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("コンポーネントIDが重複"));
    }

    // 注: test_validate_template_forces_layout_type は削除
    // LayoutType がenum化されたため、不正な値はコンパイル時に検出される

    #[test]
    fn test_get_default_template() {
        let template = get_default_template();
        assert_eq!(template.layout.layout_type, LayoutType::ThreeColumn);
        assert_eq!(template.layout.left_pct, 0.22);
        assert_eq!(template.layout.center_pct, 0.56);
        assert_eq!(template.layout.right_pct, 0.22);
        assert!(template.components.is_empty());
    }
}
