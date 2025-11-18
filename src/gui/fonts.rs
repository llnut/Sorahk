//! Windows system font loading and configuration for multi-language support.
//!
//! Loads optimized Windows fonts for English, Simplified Chinese, Traditional Chinese,
//! and Japanese. Applies vertical offset adjustments for better CJK text alignment.

use crate::i18n::Language;
use eframe::egui;
use std::sync::Arc;

/// Font configuration for Windows system fonts.
struct FontConfig {
    name: &'static str,
    path: &'static str,
}

/// Loads Windows system fonts optimized for the selected language.
///
/// Automatically loads appropriate fonts from Windows system directory and applies
/// vertical offset adjustments for better CJK text alignment in UI components.
pub fn load_fonts(ctx: &egui::Context, language: Language) {
    let mut fonts = egui::FontDefinitions::default();
    let font_configs = get_font_configs_for_language(language);
    let mut loaded_count = 0;

    let cjk_y_offset = match language {
        Language::Japanese => 0.3,
        Language::English | Language::SimplifiedChinese | Language::TraditionalChinese => 0.0,
    };

    for config in font_configs {
        if let Ok(font_data) = std::fs::read(config.path) {
            let font_data = if cjk_y_offset != 0.0 {
                egui::FontData::from_owned(font_data).tweak(egui::FontTweak {
                    y_offset_factor: cjk_y_offset,
                    ..Default::default()
                })
            } else {
                egui::FontData::from_owned(font_data)
            };

            fonts
                .font_data
                .insert(config.name.to_string(), Arc::new(font_data));

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(config.name.to_string());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(config.name.to_string());

            loaded_count += 1;
        }
    }

    let emoji_fonts = [
        ("Segoe UI Emoji", "C:\\Windows\\Fonts\\seguiemj.ttf"),
        ("Segoe UI Symbol", "C:\\Windows\\Fonts\\seguisym.ttf"),
        ("MS Gothic", "C:\\Windows\\Fonts\\msgothic.ttc"),
        ("Arial Unicode MS", "C:\\Windows\\Fonts\\arialuni.ttf"),
    ];

    for (name, path) in emoji_fonts {
        if let Ok(emoji_data) = std::fs::read(path) {
            fonts.font_data.insert(
                name.to_string(),
                Arc::new(egui::FontData::from_owned(emoji_data)),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push(name.to_string());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push(name.to_string());
        }
    }

    if loaded_count == 0 {
        eprintln!(
            "Warning: No system fonts loaded for {:?}. CJK characters may not display correctly.",
            language
        );
    }

    ctx.set_fonts(fonts);
}

/// Returns font configurations prioritized by language.
fn get_font_configs_for_language(language: Language) -> Vec<FontConfig> {
    match language {
        Language::SimplifiedChinese => vec![
            FontConfig {
                name: "Microsoft YaHei",
                path: "C:\\Windows\\Fonts\\msyh.ttc",
            },
            FontConfig {
                name: "SimHei",
                path: "C:\\Windows\\Fonts\\simhei.ttf",
            },
            FontConfig {
                name: "SimSun",
                path: "C:\\Windows\\Fonts\\simsun.ttc",
            },
        ],

        Language::TraditionalChinese => vec![
            FontConfig {
                name: "Microsoft JhengHei",
                path: "C:\\Windows\\Fonts\\msjh.ttc",
            },
            FontConfig {
                name: "MingLiU",
                path: "C:\\Windows\\Fonts\\mingliu.ttc",
            },
            FontConfig {
                name: "Microsoft YaHei",
                path: "C:\\Windows\\Fonts\\msyh.ttc",
            },
        ],

        Language::Japanese => vec![
            FontConfig {
                name: "BIZ UDGothic",
                path: "C:\\Windows\\Fonts\\BIZ-UDGothicR.ttc",
            },
            FontConfig {
                name: "Yu Gothic UI",
                path: "C:\\Windows\\Fonts\\YuGothM.ttc",
            },
            FontConfig {
                name: "Meiryo UI",
                path: "C:\\Windows\\Fonts\\meiryob.ttc",
            },
            FontConfig {
                name: "Meiryo",
                path: "C:\\Windows\\Fonts\\meiryo.ttc",
            },
        ],

        Language::English => vec![
            FontConfig {
                name: "Segoe UI",
                path: "C:\\Windows\\Fonts\\segoeui.ttf",
            },
            FontConfig {
                name: "Microsoft YaHei",
                path: "C:\\Windows\\Fonts\\msyh.ttc",
            },
        ],
    }
}
