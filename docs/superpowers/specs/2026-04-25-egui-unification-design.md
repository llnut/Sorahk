# egui GUI 统一重构设计文档

> 日期：2026-04-25
> 适用范围：`src/gui/` 子树
> 目标：在不改变功能的前提下，统一项目内 egui 写法，引入主题模块、widget 工厂模块、统一文件结构
> 不在范围：i18n 文本调整、emoji-in-format 修复（标记为后续 PR）、行为级改动

---

## 0. 背景与动机

`src/gui/` 共 13 个文件，约 11000 行。调研发现以下问题：

- **178 种独立 RGB 字面量**散布于各文件；同一语义的颜色在多处用不同取值（如 dark mode 下"标题蓝"有 3 种近似但不同的取值）
- `is_mouse_move_target` / `is_mouse_scroll_target` 在 `main_window.rs` 与 `settings_dialog/helpers.rs` **两处完全重复定义**
- 3 个 `estimate_pill_width` 变体使用 3 种冲突的 `char_width` 常量（6.5 / 7.0 / 7.5）
- 11 种字号字面量散布（11/12/13/14/15/16/18/20/22/24/28），无语义层级
- 每个对话框各自重写 `Frame::NONE.fill().corner_radius().inner_margin().shadow()` 模板
- `settings_dialog/mod.rs` 4532 行单方法、12+ 层闭包嵌套，违反 CLAUDE.md "800 行上限"约束

本文档定义一次性、可分步执行的统一方案。

## 1. 范围与决策记录

### 1.1 决策

| ID | 选项 | 决定 |
|---|---|---|
| D1 | 工作量 | **B：去重 + 拆分大文件**（不引入 widgets 组件抽象层） |
| D2 | 视觉自由度 | **C：允许我标注合并近似色，逐项审核** |
| D3 | settings_dialog 拆分维度 | **A：按功能区拆**（单层目录、文件粒度合理） |
| D4 | 共享 widget 助手 API 风格 | **路线 1：自由函数返回已配置 widget**（与 egui 原生模式一致） |
| D5 | §6 颜色合并 M1-M8 | **批量接受** |

### 1.2 不在范围

- i18n 字符串调整
- `format!("... → {}", ...)` 中的 emoji 拼接修复（违反 CLAUDE.md §8，标记为独立后续 PR）
- 任何行为级改动（包括 capture 状态机、按键映射逻辑、worker 池等）
- 添加新 crate 依赖（受 CLAUDE.md 项目约束）

## 2. 模块布局与文件树变更

新增 2 个模块（位于 `src/gui/` 平面单层），扩充 1 个、拆分 1 个、瘦身 1 个：

```
src/gui/
├── mod.rs                 (现有；轻微修改：导出 theme/widgets, 移除 cached_visuals 字段)
├── theme.rs               (新增, ~250 行) — 主题模块, 持有所有颜色 + Visuals
├── widgets.rs             (新增, ~300 行) — 自由函数: themed_button/card_frame/pill 等
├── utils.rs               (现有；扩充)    — 收纳 is_mouse_move_target 等跨模块通用 helper
├── main_window.rs         (大幅瘦身: 1664 → ~900 行)
├── about_dialog.rs        (调用 theme/widgets, 删除内联色)
├── error_dialog.rs        (同上)
├── rule_properties_dialog.rs (同上)
├── mouse_direction_dialog.rs (同上)
├── mouse_scroll_dialog.rs (同上)
├── hid_activation_dialog.rs (同上)
├── device_manager_dialog.rs (同上)
├── device_info.rs         (同上)
├── fonts.rs               (不变)
├── types.rs               (不变)
└── settings_dialog/
    ├── mod.rs             (4532 → ~400 行: 主框架 + 入口 + 顶部分发)
    ├── general.rs         (新增, ~600 行)
    ├── xinput_params.rs   (新增, ~400 行)
    ├── sequence_params.rs (新增, ~200 行)
    ├── process_list.rs    (新增, ~500 行)
    ├── mapping_list.rs    (新增, ~700 行)
    ├── mapping_editor.rs  (新增, ~800 行)
    ├── capture.rs         (现有, 扩充: 220 → ~500 行)
    └── helpers.rs         (现有, 218 → ~50 行)
```

约束达成：所有新文件 < 800 行（符合 CLAUDE.md 文件大小约束）。

## 3. `gui/theme.rs` 模块 API

### 3.1 数据模型

```rust
//! gui/theme.rs - Centralized theme: semantic colors + cached Visuals.

use eframe::egui::{self, Color32, Visuals, epaint::Shadow};

/// Semantic color roles. ~25 named slots covering all GUI surfaces.
/// All fields are `Color32` (4-byte) — the entire struct is ~100 bytes.
#[derive(Clone, Copy)]
pub struct ThemeColors {
    // Backgrounds
    pub bg_window: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_input: Color32,

    // Foregrounds
    pub fg_primary: Color32,
    pub fg_muted: Color32,
    pub fg_inverse: Color32,
    pub fg_link: Color32,

    // Title / heading
    pub title_primary: Color32,

    // Action accents
    pub accent_primary: Color32,
    pub accent_secondary: Color32,
    pub accent_danger: Color32,
    pub accent_success: Color32,
    pub accent_warning: Color32,
    pub accent_pink: Color32,

    // Pills
    pub pill_keyboard: Color32,
    pub pill_mouse_button: Color32,
    pub pill_mouse_movement: Color32,
    pub pill_gamepad: Color32,
    pub pill_target: Color32,

    // Status indicators
    pub status_active: Color32,
    pub status_paused: Color32,

    // Stroke / utility
    pub divider: Color32,
}

pub const DARK: ThemeColors = ThemeColors { /* see §6 for full values */ };
pub const LIGHT: ThemeColors = ThemeColors { /* see §6 for full values */ };

#[inline]
pub fn colors(dark_mode: bool) -> &'static ThemeColors {
    if dark_mode { &DARK } else { &LIGHT }
}

/// Owned by `SorahkGui`; pre-computed `Visuals` for both themes.
pub struct ThemeCache {
    pub dark: Visuals,
    pub light: Visuals,
}

impl ThemeCache {
    pub fn new() -> Self {
        Self {
            dark: build_dark_visuals(),
            light: build_light_visuals(),
        }
    }

    #[inline]
    pub fn visuals(&self, dark_mode: bool) -> &Visuals {
        if dark_mode { &self.dark } else { &self.light }
    }
}

fn build_dark_visuals() -> Visuals { /* moved verbatim from gui/mod.rs::create_dark_visuals */ }
fn build_light_visuals() -> Visuals { /* moved verbatim from gui/mod.rs::create_light_visuals */ }

/// Translucent overlay tints; not part of `ThemeColors` because they are
/// strictly compile-time constants used for card backgrounds.
pub mod overlay {
    use super::Color32;
    pub const PINK_TINT_DARK:  Color32 = Color32::from_rgba_premultiplied(255, 182, 193, 25);
    pub const PINK_TINT_LIGHT: Color32 = Color32::from_rgba_premultiplied(255, 218, 224, 120);
    pub const BLUE_TINT_DARK:  Color32 = Color32::from_rgba_premultiplied(135, 206, 235, 25);
    pub const BLUE_TINT_LIGHT: Color32 = Color32::from_rgba_premultiplied(173, 216, 230, 120);
    pub const SHADOW_LIGHT:    Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 25);
    pub const SHADOW_HEAVY:    Color32 = Color32::from_rgba_premultiplied(0, 0, 0, 45);
}
```

### 3.2 调用约定

```rust
// At the top of any render function (mirrors the existing `let t = self.translations;` style):
let c = theme::colors(self.dark_mode);

// Inside the function body:
ui.label(RichText::new(t.app_title()).color(c.title_primary));
```

### 3.3 SorahkGui 字段变更

```rust
// Old (gui/mod.rs):
cached_dark_visuals: egui::Visuals,
cached_light_visuals: egui::Visuals,

// New:
theme_cache: theme::ThemeCache,
```

`SorahkGui::new` 的 visuals 构造由 `theme::ThemeCache::new()` 单行替代。`create_dark_visuals` / `create_light_visuals` 两个方法删除。

### 3.4 性能要点

1. `Color32::from_rgb` 是 `const fn`（egui 0.27+），所有颜色编译期解析，零运行时构造成本
2. `colors(dark_mode)` 返回 `&'static`，内联后等价于直接访问
3. `ThemeCache.visuals(dark_mode)` 返回 `&Visuals`；每帧 `ctx.set_visuals(cache.visuals(dark_mode).clone())`，clone 是浅复制 + 几个 Vec clone（微秒级）
4. 整个 `ThemeColors` 约 100 字节，常量池放在 `.rodata`，零分配

## 4. `gui/widgets.rs` 模块 API

### 4.1 尺寸常量

```rust
//! gui/widgets.rs

/// Semantic text sizes. Replaces the 11 inline `.size(N)` literals.
pub mod text_size {
    pub const SMALL: f32 = 11.0;
    pub const COMPACT: f32 = 12.0;
    pub const BODY: f32 = 13.0;
    pub const NORMAL: f32 = 14.0;
    pub const SUBTITLE: f32 = 16.0;
    pub const SECTION: f32 = 18.0;
    pub const TITLE: f32 = 20.0;
    pub const HERO: f32 = 28.0;
}

pub mod spacing {
    pub const TIGHT: f32 = 4.0;
    pub const SMALL: f32 = 8.0;
    pub const NORMAL: f32 = 12.0;
    pub const LARGE: f32 = 16.0;
    pub const SECTION: f32 = 20.0;
}

pub mod radius {
    pub const PILL: u8 = 8;
    pub const BUTTON: u8 = 12;
    pub const CARD: u8 = 16;
    pub const DIALOG: u8 = 20;
}
```

### 4.2 按钮工厂

```rust
use eframe::egui::{self, RichText};
use crate::gui::theme;

#[derive(Clone, Copy)]
pub enum ButtonKind {
    Primary,
    Secondary,
    Danger,
    Success,
    Warning,
    Pink,
    Neutral,
}

#[inline]
pub fn themed_button(text: &str, kind: ButtonKind, dark_mode: bool) -> egui::Button<'_> {
    let c = theme::colors(dark_mode);
    let fill = match kind {
        ButtonKind::Primary => c.accent_primary,
        ButtonKind::Secondary => c.accent_secondary,
        ButtonKind::Danger => c.accent_danger,
        ButtonKind::Success => c.accent_success,
        ButtonKind::Warning => c.accent_warning,
        ButtonKind::Pink => c.accent_pink,
        ButtonKind::Neutral => c.bg_card_hover,
    };
    egui::Button::new(
        RichText::new(text).size(text_size::BODY).color(c.fg_inverse),
    )
    .fill(fill)
    .corner_radius(radius::BUTTON)
}

/// Theme toggle: gold in dark mode, lilac in light mode.
#[inline]
pub fn theme_toggle_button(text: &str, dark_mode: bool) -> egui::Button<'_> {
    let c = theme::colors(dark_mode);
    let fill = if dark_mode { c.accent_warning } else { c.accent_secondary };
    egui::Button::new(RichText::new(text).size(text_size::BODY).color(c.fg_inverse))
        .fill(fill)
        .corner_radius(radius::BUTTON)
}
```

### 4.3 Frame 工厂

```rust
use eframe::egui::{Frame, Margin, CornerRadius, epaint::Shadow};
use eframe::egui::Color32;

#[inline]
pub fn card_frame(dark_mode: bool) -> Frame {
    let c = theme::colors(dark_mode);
    Frame::NONE
        .fill(c.bg_card)
        .corner_radius(CornerRadius::same(radius::CARD))
        .inner_margin(Margin::same(spacing::LARGE as i8))
}

#[inline]
pub fn dialog_frame(dark_mode: bool) -> Frame {
    card_frame(dark_mode)
        .corner_radius(CornerRadius::same(radius::DIALOG))
        .shadow(Shadow {
            offset: [0, 4],
            blur: 18,
            spread: 0,
            color: theme::overlay::SHADOW_LIGHT,
        })
}

#[inline]
pub fn pill_frame(fill: Color32) -> Frame {
    Frame::NONE
        .fill(fill)
        .corner_radius(CornerRadius::same(radius::PILL))
        .inner_margin(Margin::symmetric(8, 4))
}
```

### 4.4 复合 widget

```rust
/// Maps a key string to its themed pill background color.
/// Replaces `helpers.rs::get_sequence_key_color` plus several inline matches.
#[inline]
pub fn pill_color(key: &str, dark_mode: bool) -> Color32 {
    let c = theme::colors(dark_mode);
    let bytes = key.as_bytes();
    let starts_with_ci = |p: &[u8]| -> bool {
        bytes.len() >= p.len() && bytes[..p.len()].eq_ignore_ascii_case(p)
    };

    if starts_with_ci(b"MOUSE_") { c.pill_mouse_movement }
    else if key.eq_ignore_ascii_case("LBUTTON")
         || key.eq_ignore_ascii_case("RBUTTON")
         || key.eq_ignore_ascii_case("MBUTTON")
         || starts_with_ci(b"XBUTTON") { c.pill_mouse_button }
    else if starts_with_ci(b"GAMEPAD_") || starts_with_ci(b"JOYSTICK_") { c.pill_gamepad }
    else { c.pill_keyboard }
}

/// Maps a key string to a 1-char display icon + label string.
/// Moved from `helpers.rs::get_sequence_key_display`.
pub fn pill_icon_and_label(key: &str) -> (&'static str, String) { /* ... */ }

/// Renders a section heading: bold text + spacing.
pub fn section_header(ui: &mut egui::Ui, text: &str, dark_mode: bool) {
    let c = theme::colors(dark_mode);
    ui.add_space(spacing::SMALL);
    ui.label(RichText::new(text).size(text_size::SECTION).strong().color(c.title_primary));
    ui.add_space(spacing::SMALL);
}

/// Pill width estimator for the read-only display in mappings card.
/// (No index, no delete button.)
#[inline]
pub fn estimate_pill_width_display(key: &str) -> f32 {
    let label_len = pill_icon_and_label(key).1.chars().count().min(20);
    16.0 + 8.0 + (label_len as f32) * 6.5
}

/// Pill width estimator for the editor (sequence editing) — has index + delete button.
#[inline]
pub fn estimate_pill_width_editor(key: &str) -> f32 {
    let label_len = pill_icon_and_label(key).1.chars().count().min(20);
    53.0 + (label_len as f32) * 6.5
}

#[inline]
pub const fn arrow_separator_width() -> f32 { 18.0 }
```

### 4.5 调用点示例

```rust
// Before (50 lines for 4 buttons):
let theme_btn = egui::Button::new(
    egui::RichText::new(theme_text).size(13.0).color(egui::Color32::WHITE),
)
.fill(if self.dark_mode {
    egui::Color32::from_rgb(255, 200, 100)
} else {
    egui::Color32::from_rgb(100, 100, 180)
})
.corner_radius(12.0);
if ui.add(theme_btn).clicked() { /* ... */ }
// (3 more buttons in same pattern)

// After (15 lines):
use crate::gui::widgets::{ButtonKind, theme_toggle_button, themed_button};
if ui.add(theme_toggle_button(theme_text, self.dark_mode)).clicked() { /* ... */ }
ui.add_space(8.0);
if ui.add(themed_button(t.settings_button(), ButtonKind::Primary, self.dark_mode)).clicked() { /* ... */ }
ui.add_space(8.0);
if ui.add(themed_button(t.devices_button(), ButtonKind::Primary, self.dark_mode)).clicked() { /* ... */ }
ui.add_space(8.0);
if ui.add(themed_button(t.about_button(), ButtonKind::Secondary, self.dark_mode)).clicked() { /* ... */ }
```

## 5. 助手函数去重 / 迁移清单

### 5.1 移到 `gui/utils.rs`

| 旧位置 | 处置 |
|---|---|
| `main_window.rs:9-42 is_mouse_move_target` | 删除；改用 `utils::is_mouse_move_target` |
| `main_window.rs:45-58 is_mouse_scroll_target` | 删除；改用 `utils::is_mouse_scroll_target` |
| `settings_dialog/helpers.rs:47-80 is_mouse_move_target` | 移到 `utils.rs::is_mouse_move_target`（pub） |
| `settings_dialog/helpers.rs:84-97 is_mouse_scroll_target` | 移到 `utils.rs::is_mouse_scroll_target`（pub） |
| `settings_dialog/helpers.rs:102-127 calculate_mouse_direction` | 移到 `utils.rs::mouse_delta_to_direction`（重命名） |

### 5.2 移到 `gui/widgets.rs`

| 旧位置 | 处置 |
|---|---|
| `settings_dialog/helpers.rs:131-152 get_sequence_key_display` | 移到 `widgets::pill_icon_and_label`，签名不变 |
| `settings_dialog/helpers.rs:156-185 get_sequence_key_color` | 删除；调用方改用 `widgets::pill_color(key, dark_mode)` |
| `settings_dialog/helpers.rs:204-210 get_target_key_color` | 删除；调用方改用 `theme::colors(dark_mode).pill_target` |
| `main_window.rs:62-68 estimate_sequence_pill_width` | 删除；改用 `widgets::estimate_pill_width_display(key)` |
| `main_window.rs:70-76 estimate_target_pill_width` | 删除；改用 `widgets::estimate_pill_width_display(key)` |
| `main_window.rs:80-82 arrow_separator_width` | 删除；改用 `widgets::arrow_separator_width()`（统一为 18.0） |
| `settings_dialog/helpers.rs:189-194 estimate_pill_width` | 移到 `widgets::estimate_pill_width_editor` |
| `settings_dialog/helpers.rs:198-200 estimate_arrow_width` | 删除；改用 `widgets::arrow_separator_width()` |
| `settings_dialog/helpers.rs:214-218 estimate_target_pill_width` | 删除；与 `estimate_pill_width_editor` 等价 |

### 5.3 移到 `gui/theme.rs`

| 旧位置 | 处置 |
|---|---|
| `gui/mod.rs:271-310 create_dark_visuals` | 移到 `theme::build_dark_visuals`（私有） |
| `gui/mod.rs:312-351 create_light_visuals` | 移到 `theme::build_light_visuals`（私有） |
| `gui/mod.rs:159-161 cached_dark/light_visuals` 字段 | 替换为 `theme_cache: theme::ThemeCache` |
| `gui/mod.rs:169-170 SorahkGui::new` 中的 visuals 构造 | 替换为 `let theme_cache = theme::ThemeCache::new();` |

### 5.4 留在原位置（settings 专用）

- `settings_dialog/helpers.rs::truncate_text_safe`
- `settings_dialog/helpers.rs::get_capture_mode_display_name`
- `settings_dialog/helpers.rs::BUTTON_TEXT_MAX_CHARS`

`helpers.rs` 最终约 50 行（仅 settings 专用）。

### 5.5 净影响估算

| 文件 | 当前 | 之后 | 增减 |
|---|---|---|---|
| `gui/main_window.rs` | 1664 | ~900 | −764 |
| `gui/settings_dialog/mod.rs` | 4532 | ~400 | −4132（搬运） |
| `gui/settings_dialog/helpers.rs` | 218 | ~50 | −168 |
| `gui/mod.rs` | 390 | ~280 | −110 |
| `gui/theme.rs`（新增） | 0 | ~250 | +250 |
| `gui/widgets.rs`（新增） | 0 | ~300 | +300 |
| `gui/utils.rs` | 191 | ~280 | +89 |
| `settings_dialog/general.rs` 等 6 新文件 | 0 | ~3200 | +3200 |
| 各对话框文件去内联色 | — | — | −400 估 |

净减少约 1700 行。

## 6. 颜色统一表（25 角色 × 2 主题 = 50 常量）

8 项合并 M1-M8 已批量接受。

### 6.1 完整取值表

```rust
pub const DARK: ThemeColors = ThemeColors {
    // Backgrounds
    bg_window: Color32::from_rgb(25, 27, 35),
    bg_card: Color32::from_rgb(38, 40, 50),
    bg_card_hover: Color32::from_rgb(48, 50, 60),
    bg_input: Color32::from_rgb(42, 44, 55),

    // Foregrounds
    fg_primary: Color32::from_rgb(220, 220, 220),
    fg_muted: Color32::from_rgb(170, 170, 190),
    fg_inverse: Color32::WHITE,
    fg_link: Color32::from_rgb(135, 206, 235),

    // Title
    title_primary: Color32::from_rgb(176, 224, 230),

    // Action accents
    accent_primary: Color32::from_rgb(135, 206, 235),
    accent_secondary: Color32::from_rgb(216, 191, 216),
    accent_danger: Color32::from_rgb(230, 100, 100),
    accent_success: Color32::from_rgb(120, 220, 140),
    accent_warning: Color32::from_rgb(255, 200, 130),
    accent_pink: Color32::from_rgb(255, 182, 193),

    // Pills
    pill_keyboard: Color32::from_rgb(255, 182, 193),
    pill_mouse_button: Color32::from_rgb(140, 180, 220),
    pill_mouse_movement: Color32::from_rgb(180, 140, 220),
    pill_gamepad: Color32::from_rgb(140, 200, 160),
    pill_target: Color32::from_rgb(135, 206, 235),

    // Status
    status_active: Color32::from_rgb(120, 220, 140),
    status_paused: Color32::from_rgb(255, 200, 130),

    // Stroke
    divider: Color32::from_rgb(60, 62, 72),
};

pub const LIGHT: ThemeColors = ThemeColors {
    bg_window: Color32::from_rgb(240, 235, 245),
    bg_card: Color32::from_rgb(250, 245, 255),
    bg_card_hover: Color32::from_rgb(245, 240, 250),
    bg_input: Color32::from_rgb(238, 233, 243),

    fg_primary: Color32::from_rgb(40, 40, 40),
    fg_muted: Color32::from_rgb(100, 100, 120),
    fg_inverse: Color32::WHITE,
    fg_link: Color32::from_rgb(70, 130, 180),

    title_primary: Color32::from_rgb(70, 130, 180),

    accent_primary: Color32::from_rgb(70, 130, 180),
    accent_secondary: Color32::from_rgb(180, 140, 220),
    accent_danger: Color32::from_rgb(220, 80, 120),
    accent_success: Color32::from_rgb(80, 180, 100),
    accent_warning: Color32::from_rgb(255, 160, 80),
    accent_pink: Color32::from_rgb(255, 150, 170),

    pill_keyboard: Color32::from_rgb(255, 210, 220),
    pill_mouse_button: Color32::from_rgb(180, 210, 255),
    pill_mouse_movement: Color32::from_rgb(220, 190, 255),
    pill_gamepad: Color32::from_rgb(180, 235, 200),
    pill_target: Color32::from_rgb(173, 216, 230),

    status_active: Color32::from_rgb(80, 180, 100),
    status_paused: Color32::from_rgb(230, 150, 50),

    divider: Color32::from_rgb(220, 220, 235),
};
```

### 6.2 已接受的合并清单（M1-M8）

| ID | 合并 | 受影响 UI |
|---|---|---|
| M1 | `(173,216,230)` 等 about/settings 标题色 → `(176,224,230)` | about / settings 对话框标题 |
| M2 | `(200,180,255)`、`(180,140,220)` → `(216,191,216)`（dark secondary） | dark mode 紫色按钮 |
| M3 | `(250,150,150)`、`(255,120,150)` → `(230,100,100)` 危险色 | dark mode 红色按钮 |
| M4 | `(140,230,150)` → `(120,220,140)` (success) | dark mode 绿色按钮 |
| M5 | `(255,200,100)` → `(255,200,130)` (warning) | dark mode pause / theme toggle |
| M6 | `(219,112,147)`、`(255,192,203)` → `(255,182,193)` (pink) | 部分粉色按钮变浅、变柔和 |
| M7 | `(180,180,200)` → `(170,170,190)` (fg_muted) | dark mode 副文本 |
| M8 | `(50,52,62)`、`(40,42,50)` → `(48,50,60)` (bg_card_hover) | hover 态背景 |

## 7. `settings_dialog` 拆分方案

### 7.1 拆分手段：跨文件多个 `impl SorahkGui` 块

Rust 允许同一类型在不同文件分开写 `impl` 块。每个新 section 文件：

```rust
// settings_dialog/general.rs
use crate::gui::SorahkGui;
use crate::config::AppConfig;
use eframe::egui;

impl SorahkGui {
    pub(super) fn render_general_section(
        &mut self,
        ui: &mut egui::Ui,
        temp_config: &mut AppConfig,
    ) { /* extracted from mod.rs */ }
}
```

`settings_dialog/mod.rs` 变成薄协调层：

```rust
impl SorahkGui {
    pub(super) fn render_settings_dialog(&mut self, ctx: &egui::Context) {
        self.poll_capture(ctx);

        let mut should_save = false;
        let mut should_cancel = false;
        let temp_config = self.temp_config.as_mut().unwrap();

        egui::Window::new("").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_general_section(ui, temp_config);
                ui.add_space(spacing::SECTION);
                self.render_xinput_params_section(ui, temp_config);
                ui.add_space(spacing::SECTION);
                self.render_sequence_params_section(ui, temp_config);
                ui.add_space(spacing::SECTION);
                self.render_process_list_section(ui, temp_config);
                ui.add_space(spacing::SECTION);
                self.render_mapping_list_section(ui, temp_config);
                ui.add_space(spacing::SECTION);
                self.render_mapping_editor_section(ui, temp_config);
            });
            (should_save, should_cancel) = self.render_footer(ui);
        });

        if should_save { self.commit_settings(); }
        if should_cancel { self.discard_settings(); }
    }
}
```

### 7.2 方法到文件的映射

| 方法 | 目标文件 | 估计行数 |
|---|---|---|
| `poll_capture` | `capture.rs`（已存在文件，扩充） | +280（capture.rs 220 → ~500） |
| `render_general_section` | `general.rs` 新增 | ~600 |
| `render_xinput_params_section` | `xinput_params.rs` 新增 | ~400 |
| `render_sequence_params_section` | `sequence_params.rs` 新增 | ~200 |
| `render_process_list_section` | `process_list.rs` 新增 | ~500 |
| `render_mapping_list_section` | `mapping_list.rs` 新增 | ~700 |
| `render_mapping_editor_section` | `mapping_editor.rs` 新增 | ~800 |
| `render_footer` + `commit_settings` + `discard_settings` | `mod.rs` orchestrator 内 | ~150 |

### 7.3 跨方法状态处理

每个 section 方法签名固定为 `(&mut self, ui: &mut egui::Ui, temp_config: &mut AppConfig)`。

需要跨 section 通信的状态（极少）通过 `self` 字段处理：
- 现有 `new_mapping_*` 系列字段、`key_capture_mode`、`sequence_capture_list` 等已经在 `SorahkGui` 上
- 现有 `xinput_params_save_pending` 已经是 `SorahkGui` 字段

只有 `should_save` / `should_cancel` 是 `render_footer` 的局部返回值（一对 bool），不污染 self。

### 7.4 借用难点处理

现有的 `finalize_sequence_capture!(self)` 宏（`mod.rs:28-38`）就是为了绕开借用冲突而存在。新 section 方法继续使用同一宏，必要时把更多操作宏化。不引入额外的状态聚合结构。

### 7.5 capture.rs 现状的复用

`mod.rs:47-330` 的 capture 主驱动逻辑全部移入 `capture.rs` 作为新方法 `poll_capture(&mut self, ctx: &egui::Context)`。`capture.rs` 220 → ~500 行。

## 8. 迁移顺序与验证策略

### 8.1 核心原则

每步结束时项目必须可编译、可测试、视觉无回归。新代码引入时旧代码暂留，杜绝半完成中间态。

### 8.2 步骤序列（10 步）

| 步 | 内容 | 编译断点 |
|---|---|---|
| **S1** | 新增 `gui/theme.rs`：ThemeColors + DARK/LIGHT + ThemeCache + build_*_visuals | 全编译通过 ✓ verify on Windows |
| **S2** | 新增 `gui/widgets.rs`：尺寸常量 + 按钮/Frame/pill 工厂（仅添加） | 全编译通过 ✓ verify on Windows |
| **S3** | 扩充 `gui/utils.rs`：搬入 is_mouse_move_target 等；helpers.rs 添加 `pub use` 临时桥接 | 全编译通过 ✓ verify on Windows |
| **S4** | 替换 `main_window.rs` 重复 helper：删除本地副本，全部改用 utils/widgets | 全编译通过 ✓ verify on Windows |
| **S5** | 迁移 6 个简单对话框：about/error/mouse_direction/mouse_scroll/rule_properties/hid_activation | 全编译通过 + 视觉抽查 ✓ verify on Windows |
| **S6** | 迁移 device_manager_dialog + device_info | 全编译通过 + 视觉抽查 ✓ verify on Windows |
| **S7** | 迁移 main_window.rs：title bar / close dialog / status / hotkey / config / mappings cards | 全编译通过 + 视觉抽查 ✓ verify on Windows |
| **S8** | 拆分 settings_dialog/mod.rs：先把 capture pipeline 抽到 capture.rs | 全编译通过 ✓ verify on Windows |
| **S9** | 拆分剩余 6 个 section（每个 section 一个子提交）：general → xinput_params → sequence_params → process_list → mapping_list → mapping_editor | 每子提交编译通过 + 视觉抽查 ✓ verify on Windows |
| **S10** | 收尾：删除桥接 `pub use`、删除已无人调用的旧函数、删除 `cached_dark/light_visuals` 字段、`cargo clippy --fix` | 全编译通过 + clippy 干净 ✓ verify on Windows |

### 8.3 每步验证流程

每步在 Windows 端必跑：

```powershell
cargo build --release
cargo test
cargo clippy -- -D warnings
```

S5/S6/S7/S9 每个子提交后做视觉抽查：
- 启动应用，切换 dark/light 主题
- 打开本步迁移过的对话框，对比按钮配色、圆角、阴影
- 触发"添加映射 → 捕获键 → 保存"完整流程，确认无功能回归

### 8.4 风险与对策

| 风险 | 对策 |
|---|---|
| 借用检查器在 settings 拆分时报错 | section 方法签名固定为 `(&mut self, ui, temp_config)`；遇阻用现有 `finalize_sequence_capture!` 同款宏拆借用 |
| 某次迁移意外改变像素布局 | S5/S6/S7 每个对话框单独子提交；视觉差立刻定位 |
| WSL 不能编译 | plan 假设 Windows 端跑验证；每个 step 末尾标注 "verify on Windows" |
| S9 一次搬太多 | 内部分 6 个子提交（一个 section 一个） |
| theme 颜色合并改变实际像素 | M1-M8 已批量接受；S5 起每对话框迁移时附带视觉对比 |
| 搬运过程遇到 `format!("... → {}", ...)` 等 emoji 拼接违规 | **原样搬运**，不在本次 PR 修，标记 TODO 留给 §9 的后续 PR 处理；避免范围蔓延 |

### 8.5 测试策略

新增测试：
- `theme.rs`: 验证 `colors(true) == &DARK`、`colors(false) == &LIGHT`
- `widgets.rs`: 验证 `pill_color("MOUSE_UP", true) == DARK.pill_mouse_movement` 等映射正确性
- `utils.rs`: 补 `test_is_mouse_move_target_all_variants`、`test_is_mouse_scroll_target_all_variants`
- `widgets.rs`: 测试 `pill_icon_and_label` 与原 `get_sequence_key_display` 行为一致

回归保护：
- 现有 `cargo test` 必须保持绿；不主动修改任何测试断言
- `state::tests` 系列测试不受 GUI 重构影响

不测试的部分：
- GUI 视觉本身（egui immediate-mode 难做 snapshot 测试，本项目无此基础设施）

## 9. 后续 PR 标记（不在本次范围）

- **emoji-in-format 修复**：`format!("... → {}", ...)` 等 5 处违反 CLAUDE.md §8。涉及 i18n 字段调整，独立 PR 处理
- **GUI 视觉 snapshot 测试基础设施**：未来如要对 immediate-mode UI 做回归保护，需评估 `egui_kittest` 或类似工具
- **mapping_editor.rs 进一步拆分**：800 行虽满足 < 800 约束但接近上限。如未来要新增 target_mode 分支，可考虑按 D3 选项 B 二次拆分

## 10. 验收标准

设计实施完成的检验点：

1. `src/gui/` 内所有 `Color32::from_rgb` 字面量数 ≤ 80（仅集中在 `theme.rs`）；其他文件无内联色
2. `is_mouse_move_target` / `is_mouse_scroll_target` / `estimate_pill_width*` / `arrow_separator_width` 在整个项目唯一定义
3. `src/gui/` 所有文件 < 800 行
4. `cargo test` 全绿
5. `cargo clippy -- -D warnings` 干净
6. Windows 端启动应用，dark/light 主题切换、所有对话框打开、添加映射全流程，无视觉回归（M1-M8 已接受合并除外）
