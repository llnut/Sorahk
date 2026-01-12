//! Tests for `CachedTranslations` and `Language`.

use super::*;

#[test]
fn test_language_default() {
    let lang = Language::default();
    assert_eq!(lang, Language::English);
}

#[test]
fn test_language_all() {
    let all_languages = Language::all();
    assert_eq!(all_languages.len(), 4);
    assert_eq!(all_languages[0], Language::English);
    assert_eq!(all_languages[1], Language::SimplifiedChinese);
    assert_eq!(all_languages[2], Language::TraditionalChinese);
    assert_eq!(all_languages[3], Language::Japanese);
}

#[test]
fn test_language_display_names() {
    assert_eq!(Language::English.display_name(), "English");
    assert_eq!(Language::SimplifiedChinese.display_name(), "简体中文");
    assert_eq!(Language::TraditionalChinese.display_name(), "繁體中文");
    assert_eq!(Language::Japanese.display_name(), "日本語");
}

#[test]
fn test_cached_translations_english() {
    let translations = CachedTranslations::new(Language::English);

    assert!(translations.app_title().contains("Sorahk"));
    assert_eq!(translations.settings_button(), "⚙  Settings");
    assert_eq!(translations.about_button(), "❤  About");
    assert_eq!(translations.dark_theme(), "🌙  Dark");
    assert_eq!(translations.light_theme(), "☀  Light");
    assert_eq!(translations.paused_status(), "Paused");
    assert_eq!(translations.running_status(), "Running");
}

#[test]
fn test_cached_translations_simplified_chinese() {
    let translations = CachedTranslations::new(Language::SimplifiedChinese);

    assert!(translations.app_title().contains("Sorahk"));
    assert_eq!(translations.settings_button(), "⚙  设置");
    assert_eq!(translations.about_button(), "❤  关于");
    assert_eq!(translations.dark_theme(), "🌙  深色");
    assert_eq!(translations.light_theme(), "☀  浅色");
    assert_eq!(translations.paused_status(), "已暂停");
    assert_eq!(translations.running_status(), "连发中");
}

#[test]
fn test_cached_translations_traditional_chinese() {
    let translations = CachedTranslations::new(Language::TraditionalChinese);

    assert!(translations.app_title().contains("Sorahk"));
    assert_eq!(translations.settings_button(), "⚙  設定");
    assert_eq!(translations.about_button(), "❤  關於");
    assert_eq!(translations.dark_theme(), "🌙  深色");
    assert_eq!(translations.light_theme(), "☀  淺色");
    assert_eq!(translations.paused_status(), "已暫停");
    assert_eq!(translations.running_status(), "連發中");
}

#[test]
fn test_cached_translations_japanese() {
    let translations = CachedTranslations::new(Language::Japanese);

    assert!(translations.app_title().contains("Sorahk"));
    assert_eq!(translations.settings_button(), "⚙  設定");
    assert_eq!(translations.about_button(), "❤  概要");
    assert_eq!(translations.dark_theme(), "🌙  ダーク");
    assert_eq!(translations.light_theme(), "☀  ライト");
    assert_eq!(translations.paused_status(), "一時停止中");
    assert_eq!(translations.running_status(), "連打中");
}

#[test]
fn test_button_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(translations.pause_button(), "⏸  Pause");
    assert_eq!(translations.start_button(), "▶  Start");
    assert_eq!(translations.exit_button(), "✕  Exit");
    assert_eq!(translations.save(), "💾  Save Settings");
    assert_eq!(translations.cancel(), "↩  Cancel");
}

#[test]
fn test_dialog_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(
        translations.settings_dialog_title(),
        "⚙ Settings ~ Configuration Panel ~"
    );
    assert_eq!(translations.close_window_title(), "💫 Close Window");
    assert_eq!(translations.error_title(), "❌ Configuration Error");
    assert_eq!(translations.close_subtitle(), "What would you like to do?");
}

#[test]
fn test_action_button_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(
        translations.minimize_to_tray_button(),
        "🗕  Minimize to Tray"
    );
    assert_eq!(translations.exit_program_button(), "🚪  Exit Program");
    assert_eq!(translations.cancel_close_button(), "↩  Cancel");
}

#[test]
fn test_error_message_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(
        translations.duplicate_trigger_error(),
        "⚠ This trigger key already exists!"
    );
}

#[test]
fn test_about_dialog_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert!(translations.about_version().contains("Version"));
    assert_eq!(
        translations.about_description_line1(),
        "A lightweight, efficient auto key press tool"
    );
    assert_eq!(translations.about_author(), "👤 Author:");
    assert_eq!(translations.about_github(), "🔗 GitHub:");
    assert_eq!(translations.about_license(), "📜 License:");
    assert_eq!(translations.about_mit_license(), "MIT License");
}

#[test]
fn test_settings_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(translations.language(), "Language:");
    assert_eq!(translations.dark_mode(), "Dark Mode:");
    assert_eq!(translations.always_on_top(), "Always on Top:");
    assert_eq!(translations.show_tray_icon(), "Show Tray Icon:");
    assert_eq!(translations.show_notifications(), "Show Notifications:");
}

#[test]
fn test_configuration_labels() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(translations.input_timeout_label(), "Input Timeout (ms):");
    assert_eq!(
        translations.default_interval_label(),
        "Default Interval (ms):"
    );
    assert_eq!(
        translations.default_duration_label(),
        "Default Duration (ms):"
    );
    assert_eq!(translations.worker_count_label(), "Worker Count:");
}

#[test]
fn test_process_whitelist_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(
        translations.process_whitelist_hint(),
        "Process Whitelist (Empty = All Enabled)"
    );
    assert_eq!(translations.process_example(), "e.g., notepad.exe");
    assert_eq!(translations.browse_button(), "🗂  Browse");
}

#[test]
fn test_yes_no_translations() {
    let en = CachedTranslations::new(Language::English);
    assert_eq!(en.yes(), "Yes");
    assert_eq!(en.no(), "No");

    let zh_cn = CachedTranslations::new(Language::SimplifiedChinese);
    assert_eq!(zh_cn.yes(), "是");
    assert_eq!(zh_cn.no(), "否");

    let zh_tw = CachedTranslations::new(Language::TraditionalChinese);
    assert_eq!(zh_tw.yes(), "是");
    assert_eq!(zh_tw.no(), "否");

    let ja = CachedTranslations::new(Language::Japanese);
    assert_eq!(ja.yes(), "はい");
    assert_eq!(ja.no(), "いいえ");
}

#[test]
fn test_format_worker_count() {
    let translations = CachedTranslations::new(Language::English);
    let formatted = translations.format_worker_count(4);
    assert!(formatted.contains("4"));
    assert!(formatted.contains("Worker Count"));
}

#[test]
fn test_translation_consistency_across_languages() {
    let languages = vec![
        Language::English,
        Language::SimplifiedChinese,
        Language::TraditionalChinese,
        Language::Japanese,
    ];

    for lang in languages {
        let trans = CachedTranslations::new(lang);

        assert!(!trans.app_title().is_empty());
        assert!(!trans.settings_button().is_empty());
        assert!(!trans.about_button().is_empty());
        assert!(!trans.pause_button().is_empty());
        assert!(!trans.start_button().is_empty());
        assert!(!trans.exit_button().is_empty());
    }
}

#[test]
fn test_cached_translations_cloning() {
    let original = CachedTranslations::new(Language::English);
    let cloned = original.clone();

    assert_eq!(original.app_title(), cloned.app_title());
    assert_eq!(original.settings_button(), cloned.settings_button());
}

#[test]
fn test_hotkey_settings_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(translations.hotkey_settings_title(), "⌨ Hotkey Settings");
    assert_eq!(translations.toggle_key_label(), "Toggle Key:");
    assert_eq!(translations.click_to_set(), "Click to set key");
    assert_eq!(translations.press_any_key(), "Press any key…");
}

#[test]
fn test_config_settings_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(translations.config_settings_title(), "⚙ Config Settings");
    assert_eq!(translations.global_config_title(), "⚙ Global Configuration");
    assert_eq!(translations.key_mappings_title(), "🎯 Key Mappings");
}

#[test]
fn test_add_mapping_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(translations.add_new_mapping_title(), "➕ Add New Mapping");
    assert_eq!(translations.add_button_text(), "+ Add");
}

#[test]
fn test_short_form_translations() {
    let translations = CachedTranslations::new(Language::English);

    assert_eq!(translations.trigger_short(), "🎯 Trigger:");
    assert_eq!(translations.target_short(), "🎮 Target:");
    assert_eq!(translations.interval_short(), "⏱ Int:");
    assert_eq!(translations.duration_short(), "⏳ Dur:");
}

#[test]
fn test_changes_hint_translation() {
    let translations = CachedTranslations::new(Language::English);
    assert_eq!(
        translations.changes_take_effect_hint(),
        "* Settings will take effect immediately after saving"
    );
}

#[test]
fn test_language_equality() {
    assert_eq!(Language::English, Language::English);
    assert_ne!(Language::English, Language::SimplifiedChinese);
    assert_ne!(Language::SimplifiedChinese, Language::TraditionalChinese);
    assert_ne!(Language::TraditionalChinese, Language::Japanese);
}

#[test]
fn test_all_translations_present() {
    let languages = Language::all();

    for lang in languages {
        let trans = CachedTranslations::new(*lang);

        assert!(
            !trans.app_title().is_empty(),
            "Missing app_title for {:?}",
            lang
        );
        assert!(
            !trans.status_title().is_empty(),
            "Missing status_title for {:?}",
            lang
        );
        assert!(
            !trans.key_mappings_title().is_empty(),
            "Missing key_mappings_title for {:?}",
            lang
        );
        assert!(
            !trans.settings_dialog_title().is_empty(),
            "Missing settings_dialog_title for {:?}",
            lang
        );
    }
}
