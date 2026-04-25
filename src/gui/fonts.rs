//! Windows CJK font loading.
//!
//! Every CJK stack loads up front so the language picker can render
//! names in all locales side by side regardless of the active UI
//! language.

use eframe::egui;
use std::sync::Arc;

/// A logical font slot and its Windows system font candidates in
/// priority order.
struct FontGroup {
    name: &'static str,
    candidates: &'static [&'static str],
}

/// Registers the Chinese, Japanese, and Korean stacks plus emoji
/// fallbacks on the egui context. The `language` argument is unused:
/// font coverage stays identical across locales so strings in any
/// language render correctly regardless of the active UI language.
pub fn load_fonts(ctx: &egui::Context, _language: crate::i18n::Language) {
    let mut fonts = egui::FontDefinitions::default();

    let groups: &[FontGroup] = &[
        FontGroup {
            name: "cjk_zh",
            candidates: &[
                r"C:\Windows\Fonts\msyh.ttc",
                r"C:\Windows\Fonts\msjh.ttc",
                r"C:\Windows\Fonts\simsun.ttc",
                r"C:\Windows\Fonts\simhei.ttf",
            ],
        },
        FontGroup {
            name: "cjk_ja",
            candidates: &[
                r"C:\Windows\Fonts\meiryo.ttc",
                r"C:\Windows\Fonts\YuGothR.ttc",
                r"C:\Windows\Fonts\YuGothM.ttc",
                r"C:\Windows\Fonts\msgothic.ttc",
                r"C:\Windows\Fonts\BIZ-UDGothicR.ttc",
            ],
        },
        FontGroup {
            name: "cjk_ko",
            candidates: &[
                r"C:\Windows\Fonts\malgun.ttf",
                r"C:\Windows\Fonts\malgunbd.ttf",
                r"C:\Windows\Fonts\gulim.ttc",
                r"C:\Windows\Fonts\batang.ttc",
            ],
        },
    ];

    let mut loaded_any = false;

    for group in groups {
        for path in group.candidates {
            if let Ok(bytes) = std::fs::read(path) {
                register(&mut fonts, group.name, bytes);
                loaded_any = true;
                break;
            }
        }
    }

    // Emoji and symbol fallbacks go last so the CJK stacks claim code
    // points they cover first.
    let emoji_fonts: &[(&str, &str)] = &[
        ("seg_emoji", r"C:\Windows\Fonts\seguiemj.ttf"),
        ("seg_symbol", r"C:\Windows\Fonts\seguisym.ttf"),
    ];

    for (name, path) in emoji_fonts {
        if let Ok(bytes) = std::fs::read(path) {
            register(&mut fonts, name, bytes);
        }
    }

    if !loaded_any {
        eprintln!(
            "Warning: no CJK system fonts loaded. Chinese, Japanese, and Korean text may render as tofu boxes."
        );
    }

    ctx.set_fonts(fonts);
}

/// Inserts `bytes` under `name` and appends the name to both font
/// families as a fallback.
fn register(fonts: &mut egui::FontDefinitions, name: &str, bytes: Vec<u8>) {
    fonts
        .font_data
        .insert(name.to_owned(), Arc::new(egui::FontData::from_owned(bytes)));

    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push(name.to_owned());
    }
}
