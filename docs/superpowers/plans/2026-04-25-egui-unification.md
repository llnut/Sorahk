# egui GUI 统一重构 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在不改变功能的前提下，把 `src/gui/` 内 178 种散落的 RGB 字面量、3 处冲突的 helper 副本、4532 行的单方法 settings_dialog/mod.rs 统一为：(1) 集中的 `theme.rs` + `widgets.rs` 模块；(2) 跨文件去重的 `utils.rs`；(3) 拆分到 6 个 sub-section 的 settings_dialog。所有新文件 < 800 行。

**Architecture:** 自由函数返回已配置 egui widget（不引入扩展 trait 或样式 mutation）。每个新 section 是同一类型的独立 `impl SorahkGui` 块。颜色编译期 `const`，每帧零分配查找。详见 spec：`docs/superpowers/specs/2026-04-25-egui-unification-design.md`。

**Tech Stack:** Rust + egui (eframe) + scc + smallvec。Windows-only 编译（WSL 编译会失败，必须在 Windows 端验证）。

---

## 重要约定

1. **每次 cargo 命令都在 Windows 端跑**（WSL 编译失败是预期）。每个 task 末尾的 "verify" 步骤等同于"在 Windows 上跑 cargo build/test/clippy 并目测对应 UI"。
2. **TDD 仅适用于纯函数**（pill_color, estimate_*, pill_icon_and_label, is_mouse_*）。Builder 工厂（themed_button, card_frame）和渲染辅助（section_header）由编译 + 视觉验证保护。
3. **先添加新代码，再删除旧代码**。中间态可以双份共存，避免半完成断点。
4. **完整 spec 引用**：每个 task 列出 spec 中对应章节，subagent 读 spec 即可获得全部上下文。
5. **Tasks 20-24 是 Task 19 的同模板复制**（拆分一个 section）。所以 Tasks 20-24 仅写差异部分（grep 模式 + section 名）；执行时把 Task 19 的完整 7 步流程套用即可。
6. **section 边界识别**：4532 行的 `settings_dialog/mod.rs` 是单方法 + 12+ 层嵌套闭包。每个 section 拆分 task 提供 grep 模式（用于定位起点）+ 文本特征（如 `t.section_general()` 之类的 i18n 调用作为边界标记）。不依赖固定行号——前序 task 已改过文件。

---

## Task 1: 创建 `gui/theme.rs` 骨架与 ThemeColors 结构

**Files:**
- Create: `src/gui/theme.rs`
- Modify: `src/gui/mod.rs`（添加 `pub mod theme;` 声明）

**Spec reference:** §3.1, §6.1

- [ ] **Step 1: 创建 `src/gui/theme.rs` 仅含结构定义**

```rust
//! gui/theme.rs - Centralized theme: semantic colors + cached Visuals.

use eframe::egui::{self, Color32, Visuals, epaint::Shadow};

/// Semantic color roles. Each field is a `Color32` (4 bytes).
/// The whole struct is ~100 bytes and lives in `.rodata`.
#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg_window: Color32,
    pub bg_card: Color32,
    pub bg_card_hover: Color32,
    pub bg_input: Color32,

    pub fg_primary: Color32,
    pub fg_muted: Color32,
    pub fg_inverse: Color32,
    pub fg_link: Color32,

    pub title_primary: Color32,

    pub accent_primary: Color32,
    pub accent_secondary: Color32,
    pub accent_danger: Color32,
    pub accent_success: Color32,
    pub accent_warning: Color32,
    pub accent_pink: Color32,

    pub pill_keyboard: Color32,
    pub pill_mouse_button: Color32,
    pub pill_mouse_movement: Color32,
    pub pill_gamepad: Color32,
    pub pill_target: Color32,

    pub status_active: Color32,
    pub status_paused: Color32,

    pub divider: Color32,
}
```

- [ ] **Step 2: 添加 DARK 常量（spec §6.1 全部值）**

在同一文件追加：

```rust
pub const DARK: ThemeColors = ThemeColors {
    bg_window: Color32::from_rgb(25, 27, 35),
    bg_card: Color32::from_rgb(38, 40, 50),
    bg_card_hover: Color32::from_rgb(48, 50, 60),
    bg_input: Color32::from_rgb(42, 44, 55),

    fg_primary: Color32::from_rgb(220, 220, 220),
    fg_muted: Color32::from_rgb(170, 170, 190),
    fg_inverse: Color32::WHITE,
    fg_link: Color32::from_rgb(135, 206, 235),

    title_primary: Color32::from_rgb(176, 224, 230),

    accent_primary: Color32::from_rgb(135, 206, 235),
    accent_secondary: Color32::from_rgb(216, 191, 216),
    accent_danger: Color32::from_rgb(230, 100, 100),
    accent_success: Color32::from_rgb(120, 220, 140),
    accent_warning: Color32::from_rgb(255, 200, 130),
    accent_pink: Color32::from_rgb(255, 182, 193),

    pill_keyboard: Color32::from_rgb(255, 182, 193),
    pill_mouse_button: Color32::from_rgb(140, 180, 220),
    pill_mouse_movement: Color32::from_rgb(180, 140, 220),
    pill_gamepad: Color32::from_rgb(140, 200, 160),
    pill_target: Color32::from_rgb(135, 206, 235),

    status_active: Color32::from_rgb(120, 220, 140),
    status_paused: Color32::from_rgb(255, 200, 130),

    divider: Color32::from_rgb(60, 62, 72),
};
```

- [ ] **Step 3: 添加 LIGHT 常量（spec §6.1 全部值）**

```rust
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

#[inline]
pub fn colors(dark_mode: bool) -> &'static ThemeColors {
    if dark_mode { &DARK } else { &LIGHT }
}
```

- [ ] **Step 4: 在 `src/gui/mod.rs` 第 6-18 行的模块声明列表里加入 `mod theme;`**

找到这段：
```rust
mod about_dialog;
mod device_info;
pub mod device_manager_dialog;
mod error_dialog;
mod fonts;
mod hid_activation_dialog;
mod main_window;
mod mouse_direction_dialog;
mod mouse_scroll_dialog;
mod rule_properties_dialog;
mod settings_dialog;
mod types;
mod utils;
```
按字典序在合适位置插入：
```rust
mod theme;
```

- [ ] **Step 5: 添加 colors() 单元测试**

在 `src/gui/theme.rs` 末尾追加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_dispatches_by_dark_mode() {
        assert!(std::ptr::eq(colors(true), &DARK));
        assert!(std::ptr::eq(colors(false), &LIGHT));
    }

    #[test]
    fn dark_and_light_have_distinct_window_fills() {
        assert_ne!(DARK.bg_window, LIGHT.bg_window);
    }
}
```

- [ ] **Step 6: Verify on Windows**

```powershell
cargo test --lib gui::theme
cargo build --release
```
Expected: 2 tests pass, build succeeds.

- [ ] **Step 7: Commit**

```bash
git add src/gui/theme.rs src/gui/mod.rs
git commit -m "feat(gui): add theme.rs with semantic color roles

Introduces ThemeColors struct with 23 named color slots and DARK/LIGHT
const instances. Pure addition: no callers yet, no behavior change.
"
```

---

## Task 2: 添加 `theme::overlay` 半透明常量

**Files:**
- Modify: `src/gui/theme.rs`

**Spec reference:** §3.1, §6.8

- [ ] **Step 1: 在 colors() 函数下方追加 overlay 子模块**

```rust
/// Translucent overlay tints. Compile-time constants used as faint
/// background fills (e.g. trigger/target highlight cards).
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

- [ ] **Step 2: Verify on Windows**

```powershell
cargo build --release
```
Expected: build succeeds.

- [ ] **Step 3: Commit**

```bash
git add src/gui/theme.rs
git commit -m "feat(gui): add theme::overlay translucent tint constants"
```

---

## Task 3: 把 `create_*_visuals` 从 `gui/mod.rs` 搬到 `theme.rs` 并加 `ThemeCache`

**Files:**
- Modify: `src/gui/theme.rs`
- Modify: `src/gui/mod.rs`

**Spec reference:** §3.1, §3.3

- [ ] **Step 1: 把 `gui/mod.rs:271-310 create_dark_visuals` 整段函数体移到 `theme.rs`**

在 `theme.rs` 末尾（在测试 mod 之前）追加，**函数从 pub fn 改为私有 fn**：

```rust
fn build_dark_visuals() -> Visuals {
    let mut visuals = egui::Visuals::dark();

    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(12);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(18);

    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    visuals.selection.stroke.width = 0.0;

    visuals.window_fill = DARK.bg_window;
    visuals.panel_fill = Color32::from_rgb(30, 32, 40);
    visuals.faint_bg_color = Color32::from_rgb(35, 37, 45);
    visuals.widgets.noninteractive.weak_bg_fill = DARK.bg_card;
    visuals.extreme_bg_color = DARK.bg_input;

    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 18,
        spread: 0,
        color: overlay::SHADOW_LIGHT,
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 3],
        blur: 12,
        spread: 0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 20),
    };

    visuals
}
```

注意：用 `DARK.bg_window` 等替换原来的 `Color32::from_rgb(...)` 字面量（颜色值已等价）。

- [ ] **Step 2: 同样移 `create_light_visuals` 到 `build_light_visuals`**

```rust
fn build_light_visuals() -> Visuals {
    let mut visuals = egui::Visuals::light();

    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(12);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(18);

    visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
    visuals.selection.stroke.width = 0.0;

    visuals.window_fill = LIGHT.bg_window;
    visuals.panel_fill = LIGHT.bg_input;
    visuals.faint_bg_color = LIGHT.bg_card_hover;
    visuals.widgets.noninteractive.weak_bg_fill = LIGHT.bg_card;
    visuals.extreme_bg_color = Color32::from_rgb(235, 230, 245);

    visuals.window_shadow = Shadow {
        offset: [0, 4],
        blur: 18,
        spread: 0,
        color: overlay::SHADOW_LIGHT,
    };
    visuals.popup_shadow = Shadow {
        offset: [0, 3],
        blur: 12,
        spread: 0,
        color: Color32::from_rgba_premultiplied(0, 0, 0, 20),
    };

    visuals
}
```

- [ ] **Step 3: 添加 `ThemeCache` 类型**

```rust
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

impl Default for ThemeCache {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: 修改 `gui/mod.rs::SorahkGui` 字段定义（第 158-161 行）**

把：
```rust
    /// Cached dark theme visuals
    cached_dark_visuals: egui::Visuals,
    /// Cached light theme visuals
    cached_light_visuals: egui::Visuals,
```
改为：
```rust
    /// Pre-computed dark/light theme visuals.
    theme_cache: theme::ThemeCache,
```

- [ ] **Step 5: 修改 `SorahkGui::new`（约第 169-170 行）**

把：
```rust
        let cached_dark_visuals = Self::create_dark_visuals();
        let cached_light_visuals = Self::create_light_visuals();
```
改为：
```rust
        let theme_cache = theme::ThemeCache::new();
```

并把构造体里：
```rust
            cached_dark_visuals,
            cached_light_visuals,
```
改为：
```rust
            theme_cache,
```

- [ ] **Step 6: 删除 `gui/mod.rs::create_dark_visuals` 与 `create_light_visuals` 两个方法（约第 271-351 行）**

整段删除（这两个方法已被 `theme::build_*_visuals` 替代）。

- [ ] **Step 7: 找到 `gui/mod.rs` 中所有读取 `cached_dark_visuals` / `cached_light_visuals` 的地方，替换为 `theme_cache.visuals(dark_mode).clone()` 或 `&theme_cache.dark/light`**

```bash
grep -n "cached_dark_visuals\|cached_light_visuals" src/gui/
```
对每处：
```rust
// before
ctx.set_visuals(self.cached_dark_visuals.clone());
// after
ctx.set_visuals(self.theme_cache.dark.clone());
```
或更通用：
```rust
ctx.set_visuals(self.theme_cache.visuals(self.dark_mode).clone());
```

- [ ] **Step 8: Verify on Windows**

```powershell
cargo test --lib gui::theme
cargo build --release
cargo clippy -- -D warnings
```
Expected: 2 theme tests pass, build succeeds, no warnings.

肉眼验证：启动 sorahk.exe，dark/light 切换正常，无视觉差异。

- [ ] **Step 9: Commit**

```bash
git add src/gui/theme.rs src/gui/mod.rs
git commit -m "refactor(gui): move Visuals construction from mod.rs to theme::ThemeCache

create_dark_visuals / create_light_visuals consolidated into private
theme::build_*_visuals. SorahkGui now holds a single theme_cache field
instead of two cached_*_visuals. Behavior unchanged."
```

---

## Task 4: 创建 `gui/widgets.rs` 骨架与尺寸常量

**Files:**
- Create: `src/gui/widgets.rs`
- Modify: `src/gui/mod.rs`

**Spec reference:** §4.1

- [ ] **Step 1: 创建 `src/gui/widgets.rs`，仅含尺寸常量**

```rust
//! gui/widgets.rs - Themed egui widget factories and dimension constants.

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

- [ ] **Step 2: 在 `src/gui/mod.rs` 模块声明列表加入 `mod widgets;`**

按字典序加入（在 `mod utils;` 之前或之后皆可）：
```rust
mod widgets;
```

- [ ] **Step 3: Verify on Windows**

```powershell
cargo build --release
```
Expected: 因为 widgets.rs 仅有常量声明且未使用，会出 dead_code 警告。这没关系——下个 task 会用上。先确认 build 成功即可。

- [ ] **Step 4: Commit**

```bash
git add src/gui/widgets.rs src/gui/mod.rs
git commit -m "feat(gui): scaffold widgets.rs with size/spacing/radius constants"
```

---

## Task 5: `widgets.rs` 添加按钮工厂

**Files:**
- Modify: `src/gui/widgets.rs`

**Spec reference:** §4.2

- [ ] **Step 1: 在 widgets.rs 文件顶部添加 use 与 ButtonKind enum**

把当前的 `//! gui/widgets.rs - ...` 注释保留，紧接着加：

```rust
use eframe::egui::{self, RichText};
use crate::gui::theme;
```

在 `pub mod radius { ... }` 块后追加：

```rust
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
        RichText::new(text.to_string()).size(text_size::BODY).color(c.fg_inverse),
    )
    .fill(fill)
    .corner_radius(radius::BUTTON)
}

#[inline]
pub fn theme_toggle_button(text: &str, dark_mode: bool) -> egui::Button<'_> {
    let c = theme::colors(dark_mode);
    let fill = if dark_mode { c.accent_warning } else { c.accent_secondary };
    egui::Button::new(
        RichText::new(text.to_string()).size(text_size::BODY).color(c.fg_inverse),
    )
    .fill(fill)
    .corner_radius(radius::BUTTON)
}
```

- [ ] **Step 2: Verify on Windows**

```powershell
cargo build --release
```
Expected: build 成功，可能有 dead_code 警告（尚无调用方）—— 暂忽略。

- [ ] **Step 3: Commit**

```bash
git add src/gui/widgets.rs
git commit -m "feat(gui): add themed_button / theme_toggle_button factories"
```

---

## Task 6: `widgets.rs` 添加 Frame 工厂

**Files:**
- Modify: `src/gui/widgets.rs`

**Spec reference:** §4.3

- [ ] **Step 1: 在文件顶部 use 处补齐 import**

在现有 `use` 行下追加：
```rust
use eframe::egui::{Color32, Frame, Margin, CornerRadius, epaint::Shadow};
```

- [ ] **Step 2: 在 themed_button 后追加三个 Frame 工厂**

```rust
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

- [ ] **Step 3: Verify on Windows**

```powershell
cargo build --release
```
Expected: build 成功。

- [ ] **Step 4: Commit**

```bash
git add src/gui/widgets.rs
git commit -m "feat(gui): add card_frame / dialog_frame / pill_frame factories"
```

---

## Task 7: `widgets.rs` 添加 pill 复合 widget（TDD）

**Files:**
- Modify: `src/gui/widgets.rs`

**Spec reference:** §4.4

- [ ] **Step 1: 写失败的 pill_color 测试**

在 widgets.rs 末尾追加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pill_color_classifies_mouse_movement() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("MOUSE_UP", true), dark.pill_mouse_movement);
        assert_eq!(pill_color("MOUSE_DOWN_LEFT", true), dark.pill_mouse_movement);
        assert_eq!(pill_color("mouse_left", true), dark.pill_mouse_movement);
    }

    #[test]
    fn pill_color_classifies_mouse_buttons() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("LBUTTON", true), dark.pill_mouse_button);
        assert_eq!(pill_color("XBUTTON1", true), dark.pill_mouse_button);
    }

    #[test]
    fn pill_color_classifies_gamepad() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("GAMEPAD_045E_A", true), dark.pill_gamepad);
        assert_eq!(pill_color("JOYSTICK_FOO", true), dark.pill_gamepad);
    }

    #[test]
    fn pill_color_keyboard_default() {
        let dark = theme::colors(true);
        assert_eq!(pill_color("A", true), dark.pill_keyboard);
        assert_eq!(pill_color("F12", true), dark.pill_keyboard);
        assert_eq!(pill_color("LCTRL", true), dark.pill_keyboard);
    }

    #[test]
    fn pill_color_respects_theme() {
        assert_ne!(pill_color("A", true), pill_color("A", false));
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

```powershell
cargo test --lib gui::widgets::tests::pill_color_classifies_mouse_movement
```
Expected: FAIL（pill_color 未定义）。

- [ ] **Step 3: 实现 pill_color**

在 `pill_frame` 函数后追加：

```rust
/// Maps a key string to its themed pill background color.
#[inline]
pub fn pill_color(key: &str, dark_mode: bool) -> Color32 {
    let c = theme::colors(dark_mode);
    let bytes = key.as_bytes();
    let starts_with_ci = |p: &[u8]| -> bool {
        bytes.len() >= p.len() && bytes[..p.len()].eq_ignore_ascii_case(p)
    };

    if starts_with_ci(b"MOUSE_") {
        c.pill_mouse_movement
    } else if key.eq_ignore_ascii_case("LBUTTON")
        || key.eq_ignore_ascii_case("RBUTTON")
        || key.eq_ignore_ascii_case("MBUTTON")
        || starts_with_ci(b"XBUTTON")
    {
        c.pill_mouse_button
    } else if starts_with_ci(b"GAMEPAD_") || starts_with_ci(b"JOYSTICK_") {
        c.pill_gamepad
    } else {
        c.pill_keyboard
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

```powershell
cargo test --lib gui::widgets::tests
```
Expected: 5 tests pass.

- [ ] **Step 5: 添加 pill_icon_and_label（搬自 helpers.rs:131-152，签名不变）**

在 `pill_color` 后追加：

```rust
/// Maps a key string to a 1-char display icon + canonicalized label.
pub fn pill_icon_and_label(key: &str) -> (&'static str, String) {
    let upper = key.to_uppercase();
    match upper.as_str() {
        "MOUSE_UP" => ("↑", "MOUSE_UP".to_string()),
        "MOUSE_DOWN" => ("↓", "MOUSE_DOWN".to_string()),
        "MOUSE_LEFT" => ("←", "MOUSE_LEFT".to_string()),
        "MOUSE_RIGHT" => ("→", "MOUSE_RIGHT".to_string()),
        "MOUSE_UP_LEFT" | "MOUSE_UPLEFT" => ("↖", "MOUSE_UP_LEFT".to_string()),
        "MOUSE_UP_RIGHT" | "MOUSE_UPRIGHT" => ("↗", "MOUSE_UP_RIGHT".to_string()),
        "MOUSE_DOWN_LEFT" | "MOUSE_DOWNLEFT" => ("↙", "MOUSE_DOWN_LEFT".to_string()),
        "MOUSE_DOWN_RIGHT" | "MOUSE_DOWNRIGHT" => ("↘", "MOUSE_DOWN_RIGHT".to_string()),
        "LBUTTON" => ("🖱", "LBUTTON".to_string()),
        "RBUTTON" => ("🖱", "RBUTTON".to_string()),
        "MBUTTON" => ("🖱", "MBUTTON".to_string()),
        "XBUTTON1" => ("🖱", "XBUTTON1".to_string()),
        "XBUTTON2" => ("🖱", "XBUTTON2".to_string()),
        _ if upper.starts_with("GAMEPAD_") => ("🎮", key.to_string()),
        _ if upper.starts_with("JOYSTICK_") => ("🕹", key.to_string()),
        _ if upper.starts_with("HID_") => ("🎛", key.to_string()),
        _ => ("⌨", key.to_string()),
    }
}
```

- [ ] **Step 6: 添加 pill_icon_and_label 测试**

在 tests mod 内追加：

```rust
#[test]
fn pill_icon_and_label_mouse() {
    assert_eq!(pill_icon_and_label("MOUSE_UP"), ("↑", "MOUSE_UP".to_string()));
    assert_eq!(pill_icon_and_label("mouse_down_right"), ("↘", "MOUSE_DOWN_RIGHT".to_string()));
}

#[test]
fn pill_icon_and_label_gamepad() {
    assert_eq!(
        pill_icon_and_label("GAMEPAD_045E_A"),
        ("🎮", "GAMEPAD_045E_A".to_string()),
    );
}

#[test]
fn pill_icon_and_label_keyboard_fallback() {
    assert_eq!(pill_icon_and_label("F12"), ("⌨", "F12".to_string()));
}
```

- [ ] **Step 7: 添加宽度估算函数 + arrow 常量**

在 pill_icon_and_label 后追加：

```rust
/// Pill width estimator for read-only display in mappings card (no index, no delete).
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
pub const fn arrow_separator_width() -> f32 {
    18.0
}
```

- [ ] **Step 8: 添加宽度估算测试**

在 tests mod 内追加：

```rust
#[test]
fn estimate_pill_width_display_grows_with_label_length() {
    let a = estimate_pill_width_display("A");
    let b = estimate_pill_width_display("MOUSE_DOWN_RIGHT");
    assert!(b > a);
}

#[test]
fn estimate_pill_width_editor_includes_index_padding() {
    // Editor base is 53 (index + delete) vs display base 24 (16 + 8 icon)
    let key = "A";
    assert!(estimate_pill_width_editor(key) > estimate_pill_width_display(key));
}

#[test]
fn arrow_separator_width_is_const() {
    assert_eq!(arrow_separator_width(), 18.0);
}
```

- [ ] **Step 9: 添加 section_header（无单元测试，需 GUI 上下文）**

```rust
/// Render a section heading: bold themed text + spacing.
pub fn section_header(ui: &mut egui::Ui, text: &str, dark_mode: bool) {
    let c = theme::colors(dark_mode);
    ui.add_space(spacing::SMALL);
    ui.label(
        RichText::new(text.to_string())
            .size(text_size::SECTION)
            .strong()
            .color(c.title_primary),
    );
    ui.add_space(spacing::SMALL);
}
```

- [ ] **Step 10: Verify on Windows**

```powershell
cargo test --lib gui::widgets
cargo build --release
cargo clippy -- -D warnings
```
Expected: 11 widgets tests pass, no warnings.

- [ ] **Step 11: Commit**

```bash
git add src/gui/widgets.rs
git commit -m "feat(gui): add pill_color/pill_icon_and_label/estimate_pill_width helpers

Behaviour matches helpers.rs::get_sequence_key_color and friends.
TDD-tested. No callers yet — replacements happen in subsequent tasks."
```

---

## Task 8: 扩充 `gui/utils.rs` —— 搬入跨模块通用 helper

**Files:**
- Modify: `src/gui/utils.rs`
- Modify: `src/gui/settings_dialog/helpers.rs`

**Spec reference:** §5.1

- [ ] **Step 1: 把 helpers.rs 中三个函数完整搬到 utils.rs（pub）**

读 `src/gui/settings_dialog/helpers.rs` 第 47-127 行。
在 `src/gui/utils.rs` 文件末尾（在测试 mod 之前）追加：

```rust
/// Check if target key is a mouse movement action.
#[inline]
pub fn is_mouse_move_target(target: &str) -> bool {
    let upper = target.to_uppercase();
    matches!(
        upper.as_str(),
        "MOUSE_UP"
            | "MOUSE_DOWN"
            | "MOUSE_LEFT"
            | "MOUSE_RIGHT"
            | "MOUSE_UP_LEFT"
            | "MOUSE_UP_RIGHT"
            | "MOUSE_DOWN_LEFT"
            | "MOUSE_DOWN_RIGHT"
            | "MOUSEUP"
            | "MOUSEDOWN"
            | "MOUSELEFT"
            | "MOUSERIGHT"
            | "MOVE_UP"
            | "MOVE_DOWN"
            | "MOVE_LEFT"
            | "MOVE_RIGHT"
            | "M_UP"
            | "M_DOWN"
            | "M_LEFT"
            | "M_RIGHT"
            | "MOUSEUPLEFT"
            | "MOUSEUPRIGHT"
            | "MOUSEDOWNLEFT"
            | "MOUSEDOWNRIGHT"
            | "M_UP_LEFT"
            | "M_UP_RIGHT"
            | "M_DOWN_LEFT"
            | "M_DOWN_RIGHT"
    )
}

/// Check if target key is a mouse scroll action.
#[inline]
pub fn is_mouse_scroll_target(target: &str) -> bool {
    let upper = target.to_uppercase();
    matches!(
        upper.as_str(),
        "SCROLL_UP"
            | "SCROLLUP"
            | "WHEEL_UP"
            | "WHEELUP"
            | "SCROLL_DOWN"
            | "SCROLLDOWN"
            | "WHEEL_DOWN"
            | "WHEELDOWN"
    )
}

/// Converts mouse delta to 8-directional movement string.
/// Screen coordinates: Y+ = down, Y- = up.
#[inline]
pub fn mouse_delta_to_direction(delta: eframe::egui::Vec2, threshold: f32) -> Option<&'static str> {
    let mag_sq = delta.x * delta.x + delta.y * delta.y;
    if mag_sq < threshold * threshold {
        return None;
    }

    let angle = delta.y.atan2(delta.x).to_degrees();

    Some(if (-22.5..22.5).contains(&angle) {
        "MOUSE_RIGHT"
    } else if (22.5..67.5).contains(&angle) {
        "MOUSE_DOWN_RIGHT"
    } else if (67.5..112.5).contains(&angle) {
        "MOUSE_DOWN"
    } else if (112.5..157.5).contains(&angle) {
        "MOUSE_DOWN_LEFT"
    } else if !(-157.5..157.5).contains(&angle) {
        "MOUSE_LEFT"
    } else if (-157.5..-112.5).contains(&angle) {
        "MOUSE_UP_LEFT"
    } else if (-112.5..-67.5).contains(&angle) {
        "MOUSE_UP"
    } else {
        "MOUSE_UP_RIGHT"
    })
}
```

- [ ] **Step 2: 在 helpers.rs 用 `pub use` 桥接以保持调用方暂时可用**

修改 `src/gui/settings_dialog/helpers.rs` 第 47-127 行（含 `is_mouse_move_target`、`is_mouse_scroll_target`、`calculate_mouse_direction` 三个函数定义）：

**整段删除**这三个函数定义，改为：

```rust
// Re-exports from gui::utils to keep existing callers compiling during migration.
// These will be removed in Task 19 (cleanup) once all callers use utils::* directly.
pub use crate::gui::utils::{is_mouse_move_target, is_mouse_scroll_target};
pub use crate::gui::utils::mouse_delta_to_direction as calculate_mouse_direction;
```

- [ ] **Step 3: 添加 utils.rs 测试覆盖（在已有 tests mod 中追加）**

在 `src/gui/utils.rs::tests` 末尾追加：

```rust
#[test]
fn is_mouse_move_target_recognizes_all_canonical() {
    for k in ["MOUSE_UP", "MOUSE_DOWN", "MOUSE_LEFT", "MOUSE_RIGHT",
              "MOUSE_UP_LEFT", "MOUSE_UP_RIGHT", "MOUSE_DOWN_LEFT", "MOUSE_DOWN_RIGHT"] {
        assert!(is_mouse_move_target(k), "{k} should be move target");
        assert!(is_mouse_move_target(&k.to_lowercase()), "lowercase {k}");
    }
}

#[test]
fn is_mouse_move_target_rejects_buttons_and_other() {
    for k in ["LBUTTON", "RBUTTON", "A", "F1", "SCROLL_UP", ""] {
        assert!(!is_mouse_move_target(k), "{k} should NOT be move target");
    }
}

#[test]
fn is_mouse_scroll_target_recognizes_canonical() {
    for k in ["SCROLL_UP", "SCROLL_DOWN", "WHEEL_UP", "WHEEL_DOWN"] {
        assert!(is_mouse_scroll_target(k));
        assert!(is_mouse_scroll_target(&k.to_lowercase()));
    }
    assert!(!is_mouse_scroll_target("MOUSE_UP"));
    assert!(!is_mouse_scroll_target("A"));
}

#[test]
fn mouse_delta_to_direction_below_threshold_is_none() {
    let small = eframe::egui::Vec2::new(5.0, 5.0);
    assert_eq!(mouse_delta_to_direction(small, 30.0), None);
}

#[test]
fn mouse_delta_to_direction_eight_directions() {
    let r = 100.0;
    use std::f32::consts::PI;
    // Screen coords: y+ = down, so we sweep angles starting at 0=right.
    let cases = [
        (0.0_f32, "MOUSE_RIGHT"),
        (PI / 4.0, "MOUSE_DOWN_RIGHT"),
        (PI / 2.0, "MOUSE_DOWN"),
        (3.0 * PI / 4.0, "MOUSE_DOWN_LEFT"),
        (PI, "MOUSE_LEFT"),
        (-3.0 * PI / 4.0, "MOUSE_UP_LEFT"),
        (-PI / 2.0, "MOUSE_UP"),
        (-PI / 4.0, "MOUSE_UP_RIGHT"),
    ];
    for (angle, expected) in cases {
        let v = eframe::egui::Vec2::new(angle.cos() * r, angle.sin() * r);
        assert_eq!(mouse_delta_to_direction(v, 30.0), Some(expected), "angle {angle}");
    }
}
```

- [ ] **Step 4: Verify on Windows**

```powershell
cargo test --lib gui::utils
cargo build --release
cargo clippy -- -D warnings
```
Expected: utils tests pass（含原有 + 5 个新测试），build 干净。

- [ ] **Step 5: Commit**

```bash
git add src/gui/utils.rs src/gui/settings_dialog/helpers.rs
git commit -m "refactor(gui): move mouse target helpers to utils.rs

is_mouse_move_target / is_mouse_scroll_target / mouse_delta_to_direction
moved from settings_dialog/helpers.rs to gui/utils.rs (pub).
helpers.rs uses pub use bridge to keep existing callers working.
Adds 5 unit tests covering canonical names + 8 direction sweep."
```

---

## Task 9: 删除 `main_window.rs` 中的本地重复 helper

**Files:**
- Modify: `src/gui/main_window.rs`

**Spec reference:** §5.1, §5.2 第 4-7 项

- [ ] **Step 1: 删除 main_window.rs 第 9-58 行**

读 `src/gui/main_window.rs` 第 1-90 行（确认正确范围）。删除：
- 第 9-42 行：本地 `fn is_mouse_move_target`
- 第 45-58 行：本地 `fn is_mouse_scroll_target`
- 第 60-68 行：本地 `fn estimate_sequence_pill_width`
- 第 70-76 行：本地 `fn estimate_target_pill_width`
- 第 78-82 行：本地 `fn arrow_separator_width`

文件应只剩从 `/// Cached frame state to avoid repeated atomic operations.` 开始的内容。

- [ ] **Step 2: 在 main_window.rs 顶部添加 use**

在文件顶部（已有 `use` 行下方）追加：

```rust
use crate::gui::utils::{is_mouse_move_target, is_mouse_scroll_target};
use crate::gui::widgets::{
    self, ButtonKind, estimate_pill_width_display, arrow_separator_width,
};
```

- [ ] **Step 3: 替换 main_window.rs 中所有 `estimate_sequence_pill_width(...)` 和 `estimate_target_pill_width(...)` 调用**

```bash
grep -n "estimate_sequence_pill_width\|estimate_target_pill_width" src/gui/main_window.rs
```

每处都改用 `estimate_pill_width_display(...)`（spec §5.2 第 4-5 项："改用 widgets::estimate_pill_width_display"）。

- [ ] **Step 4: 替换 `arrow_separator_width()` 调用**

```bash
grep -n "arrow_separator_width" src/gui/main_window.rs
```
改用导入的 `arrow_separator_width()`（值从 20.0 → 18.0；spec §6 已接受此微调，按 §4 估算函数注释说明）。

- [ ] **Step 5: Verify on Windows**

```powershell
cargo build --release
cargo clippy -- -D warnings
```
Expected: 编译通过；不会有 "function defined but unused" 警告。

肉眼验证：启动 sorahk.exe，进入主窗口的"映射列表"，含序列触发器的 pill 排列正常（宽度估算值变化可能让换行点细微移动）。

- [ ] **Step 6: Commit**

```bash
git add src/gui/main_window.rs
git commit -m "refactor(gui): main_window.rs use shared utils/widgets helpers

Removes local duplicates of is_mouse_move/scroll_target, estimate_*_pill_width,
arrow_separator_width. Arrow width unified to 18.0 (was 20.0)."
```

---

## Task 10: 迁移 `error_dialog.rs` 用 theme + widgets

**Files:**
- Modify: `src/gui/error_dialog.rs`

**Spec reference:** §3.2, §4, §5

错误对话框是最简单的（133 行），先做这个练手。

- [ ] **Step 1: 阅读现有文件**

```bash
cat src/gui/error_dialog.rs
```

- [ ] **Step 2: 替换内联颜色**

把所有 `egui::Color32::from_rgb(...)` 字面量替换为 `c.<role>` 形式。在每个渲染函数顶部加：

```rust
use crate::gui::{theme, widgets};
// ... existing code ...

let c = theme::colors(dark_mode);  // dark_mode 来自函数参数或 visuals 推断
```

颜色映射规则（按 spec §6 的语义对照表）：
- `(255, 182, 193)` 或 `(255, 192, 203)` → `c.accent_pink`
- `(135, 206, 235)` 或 `(70, 130, 180)`（标题色） → `c.title_primary`
- `(173, 216, 230)`（标题色 light） → 同上
- `(176, 224, 230)` → `c.title_primary`
- `(220, 220, 220)`（dark 主文本） → `c.fg_primary`
- `(40, 40, 40)`（light 主文本） → `c.fg_primary`
- `(170/180, 170/180, 190/200)`（副文本） → `c.fg_muted`
- `(230, 100, 100)` 等红色 → `c.accent_danger`
- `Color32::WHITE`（按钮上的白字） → `c.fg_inverse`

不确定时去 spec §6.1 查表。

- [ ] **Step 3: 替换按钮构造为 widgets::themed_button**

把：
```rust
let btn = egui::Button::new(
    egui::RichText::new(label).size(13.0).color(egui::Color32::WHITE),
)
.fill(egui::Color32::from_rgb(255, 182, 193))
.corner_radius(12.0);
if ui.add(btn).clicked() { ... }
```
改为：
```rust
if ui.add(widgets::themed_button(label, ButtonKind::Pink, dark_mode)).clicked() { ... }
```

`use crate::gui::widgets::ButtonKind;` 加到顶部。

- [ ] **Step 4: 替换 Frame::default() 模板为 widgets::dialog_frame / card_frame**

如有：
```rust
egui::Frame::default()
    .fill(...)
    .corner_radius(...)
    .inner_margin(...)
    .shadow(...)
    .show(ui, |ui| { ... });
```
改为：
```rust
widgets::dialog_frame(dark_mode).show(ui, |ui| { ... });
```

如果原代码无 shadow（卡片而非对话框），用 `widgets::card_frame(dark_mode)`。

- [ ] **Step 5: 替换 RichText 字号字面量**

```bash
grep -n "\.size([0-9.]*)" src/gui/error_dialog.rs
```
对照 spec §4.1：
- `.size(11.0)` → `.size(text_size::SMALL)`
- `.size(13.0)` → `.size(text_size::BODY)`
- `.size(14.0)` → `.size(text_size::NORMAL)`
- `.size(16.0)` → `.size(text_size::SUBTITLE)`
- `.size(18.0)` → `.size(text_size::SECTION)`
- `.size(20.0)` → `.size(text_size::TITLE)`
- 其他映射到最近的 bucket

文件顶部加：`use crate::gui::widgets::text_size;`

- [ ] **Step 6: Verify on Windows**

```powershell
cargo build --release
cargo clippy -- -D warnings
```
Expected: build 干净。

肉眼验证：制造一个错误（如配置文件损坏）触发错误对话框；对比迁移前后视觉差异。dark/light 主题各看一次。

- [ ] **Step 7: Commit**

```bash
git add src/gui/error_dialog.rs
git commit -m "refactor(gui): error_dialog uses theme/widgets helpers"
```

---

## Task 11: 迁移 `about_dialog.rs`

**Files:**
- Modify: `src/gui/about_dialog.rs`

**与 Task 10 完全相同的流程**。规模 220 行。

- [ ] **Step 1: 阅读 src/gui/about_dialog.rs**
- [ ] **Step 2: 在渲染函数顶部加 `let c = theme::colors(dark_mode); use widgets;` import**
- [ ] **Step 3: 替换所有内联 Color32 字面量为 c.<role>（按 §6.1 映射表）**
- [ ] **Step 4: 替换按钮构造为 widgets::themed_button(text, ButtonKind::?, dark_mode)**
- [ ] **Step 5: 替换 Frame 模板为 widgets::dialog_frame / card_frame**
- [ ] **Step 6: 替换 .size(N) 字面量为 text_size::* 常量**
- [ ] **Step 7: Verify on Windows**
  - `cargo build --release && cargo clippy -- -D warnings`
  - 启动应用，打开 About 对话框；dark/light 主题各看一次
- [ ] **Step 8: Commit**
  ```bash
  git add src/gui/about_dialog.rs
  git commit -m "refactor(gui): about_dialog uses theme/widgets helpers"
  ```

---

## Task 12: 迁移 `mouse_direction_dialog.rs`

**Files:**
- Modify: `src/gui/mouse_direction_dialog.rs`

**与 Task 10 流程相同**。规模 308 行。

- [ ] **Step 1-7: 同 Task 11，应用于 mouse_direction_dialog.rs**
- [ ] **Step 8: 视觉验证**：在 settings 中触发"选择鼠标方向"，对话框 dark/light 各看一次
- [ ] **Step 9: Commit**
  ```bash
  git add src/gui/mouse_direction_dialog.rs
  git commit -m "refactor(gui): mouse_direction_dialog uses theme/widgets helpers"
  ```

---

## Task 13: 迁移 `mouse_scroll_dialog.rs`

**Files:**
- Modify: `src/gui/mouse_scroll_dialog.rs`

**与 Task 10 流程相同**。规模 221 行。

- [ ] **Step 1-7: 同 Task 11，应用于 mouse_scroll_dialog.rs**
- [ ] **Step 8: 视觉验证**：触发"选择鼠标滚轮"对话框
- [ ] **Step 9: Commit**
  ```bash
  git add src/gui/mouse_scroll_dialog.rs
  git commit -m "refactor(gui): mouse_scroll_dialog uses theme/widgets helpers"
  ```

---

## Task 14: 迁移 `rule_properties_dialog.rs`

**Files:**
- Modify: `src/gui/rule_properties_dialog.rs`

**与 Task 10 流程相同**。规模 411 行。

- [ ] **Step 1-7: 同 Task 11，应用于 rule_properties_dialog.rs**
- [ ] **Step 8: 视觉验证**：在主窗口添加映射，点 Rule Props 按钮触发对话框；测试 hold_indices checkbox + append keys 捕获
- [ ] **Step 9: Commit**
  ```bash
  git add src/gui/rule_properties_dialog.rs
  git commit -m "refactor(gui): rule_properties_dialog uses theme/widgets helpers"
  ```

---

## Task 15: 迁移 `hid_activation_dialog.rs`

**Files:**
- Modify: `src/gui/hid_activation_dialog.rs`

**与 Task 10 流程相同**。规模 434 行。

- [ ] **Step 1-7: 同 Task 11，应用于 hid_activation_dialog.rs**
- [ ] **Step 8: 视觉验证**：插入一个 HID 设备触发 activation 对话框
- [ ] **Step 9: Commit**
  ```bash
  git add src/gui/hid_activation_dialog.rs
  git commit -m "refactor(gui): hid_activation_dialog uses theme/widgets helpers"
  ```

---

## Task 16: 迁移 `device_manager_dialog.rs` + `device_info.rs`

**Files:**
- Modify: `src/gui/device_manager_dialog.rs`（1059 行）
- Modify: `src/gui/device_info.rs`（887 行）

**与 Task 10 流程相同**，但因为文件较大需更细心。两个文件可分两次提交。

### 16a. device_info.rs

- [ ] **Step 1-7: 同 Task 11，应用于 device_info.rs**
- [ ] **Step 8: Commit**
  ```bash
  git add src/gui/device_info.rs
  git commit -m "refactor(gui): device_info uses theme/widgets helpers"
  ```

### 16b. device_manager_dialog.rs

- [ ] **Step 9-15: 同 Task 11，应用于 device_manager_dialog.rs**
- [ ] **Step 16: 视觉验证**：打开 Devices 对话框；切换设备 API；调整 XInput 滑块；测试振动按钮
- [ ] **Step 17: Commit**
  ```bash
  git add src/gui/device_manager_dialog.rs
  git commit -m "refactor(gui): device_manager_dialog uses theme/widgets helpers"
  ```

---

## Task 17: 迁移 `main_window.rs` 主体

**Files:**
- Modify: `src/gui/main_window.rs`

**Spec reference:** §4.5

main_window.rs 较大（~1600 行剩余），分 4 个子提交，每个对应一个区域。

### 17a. Title bar（4 个按钮）

- [ ] **Step 1: 找到 `render_title_bar` 函数（约 720 行）**
- [ ] **Step 2: 在函数顶部加 `let c = theme::colors(self.dark_mode);`**
- [ ] **Step 3: 替换 4 个按钮（theme toggle / settings / devices / about）为 widgets::theme_toggle_button / themed_button(..., ButtonKind::Primary, ...) / themed_button(..., ButtonKind::Primary, ...) / themed_button(..., ButtonKind::Secondary, ...)**

参考 spec §4.5 的"After"代码块（settings/devices 用 Primary，about 用 Secondary）。

- [ ] **Step 4: 替换 title 标题的 RichText 颜色为 c.title_primary**
- [ ] **Step 5: Verify on Windows + commit**
  ```bash
  git add src/gui/main_window.rs
  git commit -m "refactor(gui): main_window title bar uses themed_button"
  ```

### 17b. Close confirmation dialog

- [ ] **Step 6: 找到 `render_close_dialog` 或类似（约 480 行）**
- [ ] **Step 7-9: 替换内联色 + Frame 模板 + 按钮**
- [ ] **Step 10: Verify on Windows + commit**

### 17c. Status / Hotkey / Config cards

- [ ] **Step 11: 找到 `render_status_card` / `render_hotkey_card` / `render_config_card`**
- [ ] **Step 12-14: 各 card 替换为 widgets::card_frame(self.dark_mode)；内部颜色替换**
- [ ] **Step 15: Verify on Windows + commit**

### 17d. Mappings card（含 pill 渲染）

- [ ] **Step 16: 找到 `render_mappings_card`**
- [ ] **Step 17: 把内联的 pill_color 计算（match key { ... })替换为调用 `widgets::pill_color(key, self.dark_mode)`**
- [ ] **Step 18: 替换 Frame::NONE.fill(...).corner_radius(...).inner_margin(...) 为 widgets::pill_frame(pill_color(...))**
- [ ] **Step 19: 替换其他内联色与按钮**
- [ ] **Step 20: Verify on Windows + commit**

---

## Task 18: 抽出 settings_dialog 的 capture pipeline 到 capture.rs

**Files:**
- Modify: `src/gui/settings_dialog/mod.rs`
- Modify: `src/gui/settings_dialog/capture.rs`

**Spec reference:** §7.5

- [ ] **Step 1: 阅读 settings_dialog/mod.rs 第 47-330 行（capture pipeline）**

```bash
sed -n '47,330p' src/gui/settings_dialog/mod.rs > /tmp/capture-block.rs
wc -l /tmp/capture-block.rs
```

- [ ] **Step 2: 阅读 settings_dialog/capture.rs 当前内容**

```bash
cat src/gui/settings_dialog/capture.rs
```

- [ ] **Step 3: 把 capture pipeline 整段移到 capture.rs 内的新 `impl SorahkGui` 块**

在 `capture.rs` 末尾追加：

```rust
use eframe::egui;
use crate::gui::SorahkGui;
use crate::gui::types::KeyCaptureMode;

impl SorahkGui {
    /// Drives the per-frame capture state machine. Polls keyboard / mouse /
    /// raw input and finalizes captured keys into the appropriate field on
    /// `self` (mapping target, sequence list, switch key, etc).
    pub(super) fn poll_capture(&mut self, ctx: &egui::Context) {
        // [整段从 settings_dialog/mod.rs:47-330 复制过来]
        // 注意：原代码在 render_settings_dialog 函数体内，依赖 self 字段直接访问；
        // 移到这里后只需把外层缺失的 use 补全（已加 KeyCaptureMode）。
    }
}
```

- [ ] **Step 4: 在 settings_dialog/mod.rs 中删除原 capture 段，并在 render_settings_dialog 开头调用**

把 `render_settings_dialog` 函数体最开始的整段 capture 代码（47-330 行）替换为单行：

```rust
self.poll_capture(ctx);
```

- [ ] **Step 5: Verify on Windows**

```powershell
cargo build --release
cargo test
cargo clippy -- -D warnings
```
Expected: 编译通过，所有测试绿。

肉眼验证：启动应用，进入 settings；点"捕获"按键映射；尝试捕获键盘单键、组合键、鼠标按钮、序列触发器。所有捕获路径应工作如初。

- [ ] **Step 6: Commit**

```bash
git add src/gui/settings_dialog/mod.rs src/gui/settings_dialog/capture.rs
git commit -m "refactor(gui): extract settings_dialog capture pipeline to capture.rs

The 280-line capture state machine moved from render_settings_dialog
into a dedicated SorahkGui::poll_capture method in capture.rs.
mod.rs now calls self.poll_capture(ctx) at the top of render_settings_dialog.
Behavior unchanged."
```

---

## Task 19: 拆分 settings_dialog —— general 区到 general.rs

**Files:**
- Create: `src/gui/settings_dialog/general.rs`
- Modify: `src/gui/settings_dialog/mod.rs`

**Spec reference:** §7

- [ ] **Step 1: 在 settings_dialog/mod.rs 中识别"通用设置"区域**

通用设置 = 语言/主题/托盘/最小化/自启动/切换键/序列终止键。是 settings_dialog 顶部的第一个折叠组或卡片。

```bash
grep -n "language\|tray\|switch_key\|sequence_finalize" src/gui/settings_dialog/mod.rs | head -20
```

阅读相应代码块，识别开始/结束行。

- [ ] **Step 2: 创建 general.rs**

```rust
//! gui/settings_dialog/general.rs - General settings section
//! (language, theme, tray, autostart, switch key, sequence finalize key).

use eframe::egui;
use crate::config::AppConfig;
use crate::gui::SorahkGui;

impl SorahkGui {
    pub(super) fn render_general_section(
        &mut self,
        ui: &mut egui::Ui,
        temp_config: &mut AppConfig,
    ) {
        // [搬运过来的代码]
    }
}
```

- [ ] **Step 3: 把对应代码块从 mod.rs 整段剪切到 general.rs::render_general_section 函数体内**

边界识别：通用设置区域通常在 `render_settings_dialog` 中以 `egui::CollapsingHeader::new(t.section_general())` 或类似的国际化标题开头，到下一个 `egui::CollapsingHeader` 出现前结束。如无 CollapsingHeader 包裹则按 i18n 标签文本（"语言"、"主题"、"切换键"）所在的连续代码段定位。

整段 ctrl-X 剪切到 general.rs 的 `render_general_section` 函数体内。注意需要补齐缺失的 `use` 语句（如 `use crate::config::Language;` 等），可以先 cargo build 看报错再补。

- [ ] **Step 4: 在 mod.rs 中调用新方法**

在 render_settings_dialog 中原"通用设置"代码的位置，替换为：

```rust
self.render_general_section(ui, temp_config);
ui.add_space(crate::gui::widgets::spacing::SECTION);
```

- [ ] **Step 5: 在 mod.rs 顶部声明子模块**

```rust
mod general;
```

- [ ] **Step 6: Verify on Windows**

```powershell
cargo build --release
cargo test
cargo clippy -- -D warnings
```

肉眼验证：打开 settings；调整语言、主题、tray、switch key 设置；保存；重启应用；设置应已生效。

- [ ] **Step 7: Commit**

```bash
git add src/gui/settings_dialog/general.rs src/gui/settings_dialog/mod.rs
git commit -m "refactor(gui): split settings general section into general.rs"
```

---

## Task 20: 拆分 settings_dialog —— XInput 参数到 xinput_params.rs

**Files:**
- Create: `src/gui/settings_dialog/xinput_params.rs`
- Modify: `src/gui/settings_dialog/mod.rs`

**与 Task 19 完全相同的流程**，目标 XInput 全局参数区域（死区/扳机阈值/振动等）。

- [ ] **Step 1: 在 mod.rs 中识别 XInput 参数区域**
  ```bash
  grep -n "xinput\|stick_deadzone\|trigger_threshold\|vibration" src/gui/settings_dialog/mod.rs | head -10
  ```
- [ ] **Step 2: 创建 `xinput_params.rs`，含 `impl SorahkGui::render_xinput_params_section`**
- [ ] **Step 3: 搬运代码块**
- [ ] **Step 4: 替换 mod.rs 调用为 `self.render_xinput_params_section(ui, temp_config);`**
- [ ] **Step 5: 在 mod.rs 加 `mod xinput_params;`**
- [ ] **Step 6: Verify on Windows**：调整死区滑块；振动测试；保存；持久化正常
- [ ] **Step 7: Commit**
  ```bash
  git commit -am "refactor(gui): split settings XInput params into xinput_params.rs"
  ```

---

## Task 21: 拆分 settings_dialog —— 序列匹配参数到 sequence_params.rs

**Files:**
- Create: `src/gui/settings_dialog/sequence_params.rs`
- Modify: `src/gui/settings_dialog/mod.rs`

**与 Task 19 完全相同的流程**，目标序列匹配参数（time window / dedup 阈值）。

- [ ] **Step 1-7: 同 Task 20，目标 `render_sequence_params_section`**

---

## Task 22: 拆分 settings_dialog —— 进程白名单到 process_list.rs

**Files:**
- Create: `src/gui/settings_dialog/process_list.rs`
- Modify: `src/gui/settings_dialog/mod.rs`

**与 Task 19 完全相同的流程**。

- [ ] **Step 1-7: 同 Task 20，目标 `render_process_list_section`**
- [ ] **视觉验证**：增删进程；duplicate process error 提示；保存

---

## Task 23: 拆分 settings_dialog —— 已有映射列表到 mapping_list.rs

**Files:**
- Create: `src/gui/settings_dialog/mapping_list.rs`
- Modify: `src/gui/settings_dialog/mod.rs`

**这是最大的一段**（约 700 行），需更细心。

- [ ] **Step 1: 识别"已有映射列表"代码块**

包含每条映射的展示 + 编辑展开 + Rule Props 按钮 + 删除按钮。grep 模式：
```bash
grep -n "for (idx, mapping) in temp_config.mappings\|temp_config\.mappings\.iter\|to_remove.*mapping" src/gui/settings_dialog/mod.rs | head -10
```
通常以一个对 `temp_config.mappings` 的 for 循环开头，到 `if let Some(idx) = to_remove` 处理删除结束。

- [ ] **Step 2-5: 同 Task 19 流程（创建 mapping_list.rs + 搬运 + 在 mod.rs 调用 + 加 mod 声明）**

- [ ] **Step 6: 替换搬运过来的 helpers::* 调用为 widgets::* 等价物**

```bash
grep -n "helpers::get_sequence_key_color\|helpers::get_sequence_key_display\|helpers::get_target_key_color\|helpers::estimate_pill_width\|helpers::estimate_target_pill_width\|helpers::estimate_arrow_width" src/gui/settings_dialog/mapping_list.rs
```

替换映射：
- `helpers::get_sequence_key_color(key, dark_mode)` → `widgets::pill_color(key, dark_mode)`
- `helpers::get_sequence_key_display(key)` → `widgets::pill_icon_and_label(key)`
- `helpers::get_target_key_color(dark_mode)` → `theme::colors(dark_mode).pill_target`
- `helpers::estimate_pill_width(key)` → `widgets::estimate_pill_width_editor(key)`
- `helpers::estimate_target_pill_width(key)` → `widgets::estimate_pill_width_editor(key)`
- `helpers::estimate_arrow_width()` → `widgets::arrow_separator_width()`

文件顶部 use 列表加：
```rust
use crate::gui::{theme, widgets};
```

- [ ] **Step 7: 替换搬运过来的内联 Color32 字面量为 c.<role>**（同 Task 11 §6 颜色映射表）

- [ ] **Step 8: 替换 RichText.size(N) 字面量为 text_size::* 常量**

- [ ] **Step 9: 替换按钮构造为 widgets::themed_button**

- [ ] **Step 10: Verify on Windows**

```powershell
cargo build --release
cargo test
cargo clippy -- -D warnings
```

视觉验证：
  - 编辑现有映射的 trigger / target / sequence
  - 删除映射
  - 触发 Rule Props 对话框
  - 切换 turbo

- [ ] **Step 11: Commit**

```bash
git add src/gui/settings_dialog/mapping_list.rs src/gui/settings_dialog/mod.rs
git commit -m "refactor(gui): split settings mapping list into mapping_list.rs

Extracts the existing-mapping list rendering (~700 lines) and migrates
internal helpers::* calls to widgets::* equivalents."
```

---

## Task 24: 拆分 settings_dialog —— 新建映射表单到 mapping_editor.rs

**Files:**
- Create: `src/gui/settings_dialog/mapping_editor.rs`
- Modify: `src/gui/settings_dialog/mod.rs`

**第二大的一段**（约 800 行）。

- [ ] **Step 1: 识别"新建映射"卡片代码块**

包含 trigger 输入 / target 输入 / sequence 输入 / 各 target_mode 分支 / Add 按钮。grep 模式：
```bash
grep -n "self.new_mapping_trigger\|self.new_mapping_target\|self.new_mapping_target_keys\|new_mapping_target_mode\|t\.add_mapping_button" src/gui/settings_dialog/mod.rs | head -20
```
通常以新建映射相关 self 字段的访问聚集开始，到 Add 按钮（`t.add_mapping_button()`）的处理结束。

- [ ] **Step 2-5: 同 Task 19 流程（创建 mapping_editor.rs + 搬运 + 在 mod.rs 调用 + 加 mod 声明）**

- [ ] **Step 6: 替换搬运过来的 helpers::* 调用为 widgets::* 等价物**

```bash
grep -n "helpers::get_sequence_key_color\|helpers::get_sequence_key_display\|helpers::get_target_key_color\|helpers::estimate_pill_width\|helpers::estimate_target_pill_width\|helpers::estimate_arrow_width" src/gui/settings_dialog/mapping_editor.rs
```

替换映射同 Task 23 Step 6。

- [ ] **Step 7: 替换搬运过来的内联 Color32 字面量为 c.<role>**

- [ ] **Step 8: 替换 RichText.size(N) 字面量为 text_size::* 常量**

- [ ] **Step 9: 替换按钮构造为 widgets::themed_button**

- [ ] **Step 10: Verify on Windows**

视觉验证：
  - 新建 Single 映射
  - 新建 Multi 映射
  - 新建 Sequence 映射
  - 新建 Sequence trigger
  - 在新建表单上点 Rule Props 按钮
  - duplicate mapping error 提示

- [ ] **Step 11: Commit**

```bash
git add src/gui/settings_dialog/mapping_editor.rs src/gui/settings_dialog/mod.rs
git commit -m "refactor(gui): split settings mapping editor into mapping_editor.rs

Extracts the new-mapping form (~800 lines) including all target_mode
branches and Rule Props integration. Migrates internal helpers::* calls
to widgets::* equivalents."
```

---

## Task 25: 收尾 —— 删除桥接、运行 clippy --fix

**Files:**
- Modify: `src/gui/settings_dialog/helpers.rs`
- Modify: `src/gui/main_window.rs`（如有残留 dead code）
- Modify: `src/gui/mod.rs`（如有残留）

**Spec reference:** §8.2 S10

- [ ] **Step 1: 验证 helpers.rs 中 `pub use` 桥接已无人调用**

```bash
grep -rn "settings_dialog::helpers::is_mouse_move_target\|settings_dialog::helpers::is_mouse_scroll_target\|settings_dialog::helpers::calculate_mouse_direction" src/
```
Expected: 无结果。

- [ ] **Step 2: 删除 helpers.rs 中的 pub use 桥接**

把 Task 8 添加的：
```rust
pub use crate::gui::utils::{is_mouse_move_target, is_mouse_scroll_target};
pub use crate::gui::utils::mouse_delta_to_direction as calculate_mouse_direction;
```
整段删除。

- [ ] **Step 3: 验证 helpers.rs 其他无人调用的函数也清理**

```bash
grep -rn "get_sequence_key_color\|get_sequence_key_display\|get_target_key_color\|estimate_pill_width\|estimate_arrow_width\|estimate_target_pill_width" src/
```
对每个 grep 结果，如果只在 helpers.rs 内匹配（即无外部调用），删除该函数。

- [ ] **Step 4: 运行 clippy --fix 自动清理未使用 import**

```powershell
cargo clippy --fix --allow-dirty --allow-staged
```

- [ ] **Step 5: Verify on Windows**

```powershell
cargo build --release
cargo test
cargo clippy -- -D warnings
```
Expected: 全绿。

- [ ] **Step 6: 验收检查**

按 spec §10：
1. `grep -rn "Color32::from_rgb" src/gui/ | wc -l` ≤ 80（仅 theme.rs 内 + 极少特例）
2. `grep -rn "fn is_mouse_move_target\|fn is_mouse_scroll_target\|fn estimate_pill_width\|fn arrow_separator_width" src/gui/ | wc -l` 必须 = 4（每函数恰好一处）
3. `wc -l src/gui/*.rs src/gui/settings_dialog/*.rs` 每文件 < 800

如有违反，定位并修复。

- [ ] **Step 7: 视觉全程回归**

启动应用，dark/light 主题切换。依次打开：
- 主窗口（status / hotkey / config / mappings）
- About 对话框
- Settings 对话框（每个区域 + 添加映射 + 编辑映射 + Rule Props + 进程白名单）
- Devices 对话框（XInput 滑块、振动、设备 API 切换）
- Error 对话框（人为损坏 Config.toml 触发）
- HID 激活对话框（插入 HID 设备）
- 鼠标方向 / 滚轮选择对话框

无视觉回归（除 §6 已接受的 M1-M8 外）。无功能回归。

- [ ] **Step 8: Commit**

```bash
git add -u
git commit -m "refactor(gui): cleanup pub use bridges and unused helpers

Removes the temporary pub use shims in settings_dialog/helpers.rs and
deletes pill / sequence display helpers superseded by widgets.rs.
Final state: all GUI files < 800 lines, ~50 RGB literals (all in theme.rs)."
```

---

## 验收清单（执行结束时）

- [ ] `cargo build --release` 全绿（Windows）
- [ ] `cargo test` 全绿（Windows）；新增测试至少 11（widgets）+ 5（utils）+ 2（theme）= 18 个
- [ ] `cargo clippy -- -D warnings` 干净
- [ ] `find src/gui -name "*.rs" -exec wc -l {} +` 每文件 < 800
- [ ] `grep -rn "Color32::from_rgb" src/gui/ | wc -l` ≤ 80
- [ ] 全功能视觉回归测试通过（如 Task 25 Step 7）
- [ ] git log 显示 ~25 个有意义的提交，每个对应一个 task 完成
