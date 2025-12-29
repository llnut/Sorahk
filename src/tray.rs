//! Tray icon implementation with i18n support.
//!
//! This module provides system tray functionality with the following features:
//! - Cached UTF-16 string conversions using SmallVec
//! - Inline functions for frequently called code paths
//! - Pre-allocated string buffers to reduce allocations
//! - AVX2 SIMD instructions for XML escaping when available

use windows::{
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::System::Registry::*,
    Win32::System::Threading::Sleep,
    Win32::UI::Shell::*,
    Win32::UI::WindowsAndMessaging::*,
    core::*,
};

use anyhow::{Result, anyhow};
use smallvec::SmallVec;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::i18n::{CachedTranslations, Language};
use crate::state::{NotificationEvent, get_global_state};

const TRAY_MESSAGE_ID: u32 = WM_APP + 1;
const AUMID: &str = "Sorahk.AutoKeyPress";

/// Cache for UTF-16 encoded strings
struct Utf16Cache {
    tooltip: SmallVec<[u16; 64]>,
    title: SmallVec<[u16; 128]>,
    message: SmallVec<[u16; 256]>,
}

impl Utf16Cache {
    #[inline]
    fn new() -> Self {
        Self {
            tooltip: SmallVec::new(),
            title: SmallVec::new(),
            message: SmallVec::new(),
        }
    }

    #[inline(always)]
    fn encode_tooltip(&mut self, text: &str) -> &[u16] {
        self.tooltip.clear();
        self.tooltip.extend(text.encode_utf16());
        self.tooltip.push(0);
        &self.tooltip
    }

    #[inline(always)]
    fn encode_title(&mut self, text: &str) -> &[u16] {
        self.title.clear();
        self.title.extend(text.encode_utf16());
        self.title.push(0);
        &self.title
    }

    #[inline(always)]
    fn encode_message(&mut self, text: &str) -> &[u16] {
        self.message.clear();
        self.message.extend(text.encode_utf16());
        self.message.push(0);
        &self.message
    }
}

pub struct TrayIcon {
    nid: NOTIFYICONDATAW,
    should_exit: Arc<AtomicBool>,
    utf16_cache: Utf16Cache,
    translations: CachedTranslations,
    last_language: u8,
}

impl TrayIcon {
    /// Create new tray icon with i18n support
    pub fn new(should_exit: Arc<AtomicBool>) -> Result<Self> {
        let state = get_global_state().ok_or(anyhow!("Failed to get app state"))?;
        let language = state.language();
        let language_u8 = language.to_u8();
        let translations = CachedTranslations::new(language);
        let window_class = w!("SorahkWindowClass");
        let instance = unsafe { GetModuleHandleW(None)? };

        let window_icon = unsafe {
            #[allow(clippy::manual_dangling_ptr)]
            let embedded = LoadIconW(Some(instance.into()), PCWSTR::from_raw(1 as *const u16));
            if embedded.is_ok() {
                embedded?
            } else {
                LoadIconW::<PCWSTR>(None, IDI_APPLICATION)?
            }
        };

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::window_procedure),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: window_icon,
            hCursor: unsafe { LoadCursorW::<PCWSTR>(None, IDC_ARROW)? },
            hbrBackground: unsafe { GetSysColorBrush(SYS_COLOR_INDEX(COLOR_WINDOW.0 + 1)) },
            lpszMenuName: PCWSTR::null(),
            lpszClassName: window_class,
        };

        let atom = unsafe { RegisterClassW(&wc) };
        if atom == 0 {
            return Err(Error::new(E_FAIL, "Failed to register window class").into());
        }

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                w!("Sorahk"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                Some(instance.into()),
                None,
            )
        }?;

        let initial_icon = unsafe {
            #[allow(clippy::manual_dangling_ptr)]
            let embedded_icon = LoadIconW(Some(instance.into()), PCWSTR::from_raw(1 as *const u16));
            if let Ok(icon) = embedded_icon {
                icon
            } else {
                LoadIconW::<PCWSTR>(None, IDI_APPLICATION)?
            }
        };

        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: WM_APP + 1,
            hIcon: initial_icon,
            ..Default::default()
        };

        let mut instance = Self {
            nid,
            should_exit,
            utf16_cache: Utf16Cache::new(),
            translations,
            last_language: language_u8,
        };

        let tooltip = instance.translations.app_title();
        let tooltip_utf16 = instance.utf16_cache.encode_tooltip(tooltip);
        let copy_len = tooltip_utf16.len().min(instance.nid.szTip.len());
        instance.nid.szTip[..copy_len].copy_from_slice(&tooltip_utf16[..copy_len]);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &instance.nid);
        }

        let _ = Self::register_aumid();

        Ok(instance)
    }

    /// Check and update translations if language changed
    #[inline]
    fn check_and_update_language(&mut self) {
        if let Some(state) = get_global_state() {
            let current_language = state.language().to_u8();

            if crate::util::unlikely(current_language != self.last_language) {
                self.last_language = current_language;
                let language = Language::from_u8(current_language);
                self.translations = CachedTranslations::new(language);

                let _ = self.update_tooltip();
            }
        }
    }

    /// Update tray icon tooltip
    fn update_tooltip(&mut self) -> Result<()> {
        let tooltip = self.translations.app_title();
        let tooltip_utf16 = self.utf16_cache.encode_tooltip(tooltip);
        let copy_len = tooltip_utf16.len().min(self.nid.szTip.len());
        self.nid.szTip[..copy_len].copy_from_slice(&tooltip_utf16[..copy_len]);

        unsafe {
            let original_flags = self.nid.uFlags;
            self.nid.uFlags = NIF_TIP | NIF_SHOWTIP;
            let result = Shell_NotifyIconW(NIM_MODIFY, &self.nid);
            self.nid.uFlags = original_flags;

            if !result.as_bool() {
                return Err(anyhow!("Failed to update tooltip"));
            }
        }

        Ok(())
    }

    #[inline]
    fn register_aumid() -> Result<()> {
        unsafe {
            let registry_path = format!("Software\\Classes\\AppUserModelId\\{}", AUMID);
            let registry_path_wide: SmallVec<[u16; 128]> = registry_path
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let mut hkey = HKEY::default();
            let result = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR::from_raw(registry_path_wide.as_ptr()),
                Some(0),
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_WRITE,
                None,
                &mut hkey,
                None,
            );

            if result.is_err() {
                return Err(anyhow!("Failed to create registry key"));
            }

            let display_name = "Sorahk - Auto Keypress Tool";
            let display_name_wide: SmallVec<[u16; 64]> = display_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let display_name_bytes = std::slice::from_raw_parts(
                display_name_wide.as_ptr() as *const u8,
                display_name_wide.len() * 2,
            );

            let _ = RegSetValueExW(
                hkey,
                w!("DisplayName"),
                Some(0),
                REG_SZ,
                Some(display_name_bytes),
            );

            if let Ok(exe_path) = std::env::current_exe() {
                let icon_uri = exe_path.to_string_lossy();
                let icon_uri_wide: SmallVec<[u16; 256]> =
                    icon_uri.encode_utf16().chain(std::iter::once(0)).collect();
                let icon_uri_bytes = std::slice::from_raw_parts(
                    icon_uri_wide.as_ptr() as *const u8,
                    icon_uri_wide.len() * 2,
                );

                let _ = RegSetValueExW(hkey, w!("IconUri"), Some(0), REG_SZ, Some(icon_uri_bytes));
            }

            let _ = RegCloseKey(hkey);
            Ok(())
        }
    }

    #[inline]
    fn unregister_aumid() -> Result<()> {
        unsafe {
            let registry_path = format!("Software\\Classes\\AppUserModelId\\{}", AUMID);
            let registry_path_wide: SmallVec<[u16; 128]> = registry_path
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let result = RegDeleteTreeW(
                HKEY_CURRENT_USER,
                PCWSTR::from_raw(registry_path_wide.as_ptr()),
            );

            if result.is_ok() {
                Ok(())
            } else {
                Err(anyhow!("Failed to unregister AUMID"))
            }
        }
    }

    #[inline(always)]
    fn show_notification(
        &mut self,
        title: &str,
        message: &str,
        icon_type: NOTIFY_ICON_INFOTIP_FLAGS,
    ) -> Result<()> {
        match Self::show_toast_notification(title, message) {
            Ok(_) => Ok(()),
            Err(_) => self.show_legacy_notification(title, message, icon_type),
        }
    }

    #[inline]
    fn show_toast_notification(title: &str, message: &str) -> Result<()> {
        let toast_xml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<toast duration="short">
    <visual>
        <binding template="ToastGeneric">
            <text>{}</text>
            <text>{}</text>
        </binding>
    </visual>
    <audio silent="true"/>
</toast>"#,
            Self::xml_escape_fast(title),
            Self::xml_escape_fast(message)
        );

        Self::try_show_toast_with_aumid(&toast_xml, AUMID)
    }

    #[inline]
    fn try_show_toast_with_aumid(toast_xml: &str, app_id: &str) -> Result<()> {
        let xml_doc = XmlDocument::new()?;
        xml_doc.LoadXml(&HSTRING::from(toast_xml))?;
        let toast = ToastNotification::CreateToastNotification(&xml_doc)?;
        let aumid = HSTRING::from(app_id);
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&aumid)?;
        notifier.Show(&toast)?;
        Ok(())
    }

    #[inline]
    fn show_legacy_notification(
        &mut self,
        title: &str,
        message: &str,
        icon_type: NOTIFY_ICON_INFOTIP_FLAGS,
    ) -> Result<()> {
        let original_flags = self.nid.uFlags;
        self.nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP | NIF_INFO;
        self.nid.dwInfoFlags = icon_type | NIIF_NOSOUND;

        let title_utf16 = self.utf16_cache.encode_title(title);
        let title_len = title_utf16.len().min(self.nid.szInfoTitle.len());
        self.nid.szInfoTitle[..title_len].copy_from_slice(&title_utf16[..title_len]);

        let message_utf16 = self.utf16_cache.encode_message(message);
        let message_len = message_utf16.len().min(self.nid.szInfo.len());
        self.nid.szInfo[..message_len].copy_from_slice(&message_utf16[..message_len]);

        self.nid.Anonymous = NOTIFYICONDATAW_0 { uTimeout: 5000 };

        let result = unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.nid) };

        if !result.as_bool() {
            self.nid.uFlags = original_flags;
            return Err(anyhow!("Failed to show legacy notification"));
        }

        self.nid.uFlags = original_flags;
        Ok(())
    }

    /// Escapes XML special characters, using AVX2 instructions when available
    #[inline]
    fn xml_escape_fast(s: &str) -> String {
        if s.len() < 32 {
            Self::xml_escape_scalar(s)
        } else {
            #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
            {
                Self::xml_escape_avx2(s)
            }
            #[cfg(not(all(target_arch = "x86_64", target_feature = "avx2")))]
            {
                Self::xml_escape_scalar(s)
            }
        }
    }

    #[inline(always)]
    fn xml_escape_scalar(s: &str) -> String {
        let mut result = String::with_capacity(s.len() + 16);
        for ch in s.chars() {
            match ch {
                '&' => result.push_str("&amp;"),
                '<' => result.push_str("&lt;"),
                '>' => result.push_str("&gt;"),
                '"' => result.push_str("&quot;"),
                '\'' => result.push_str("&apos;"),
                _ => result.push(ch),
            }
        }
        result
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    #[inline]
    fn xml_escape_avx2(s: &str) -> String {
        use std::arch::x86_64::*;

        let bytes = s.as_bytes();
        let mut result = String::with_capacity(s.len() + 16);
        let mut i = 0;

        unsafe {
            let amp = _mm256_set1_epi8(b'&' as i8);
            let lt = _mm256_set1_epi8(b'<' as i8);
            let gt = _mm256_set1_epi8(b'>' as i8);
            let quot = _mm256_set1_epi8(b'"' as i8);
            let apos = _mm256_set1_epi8(b'\'' as i8);

            while i + 32 <= bytes.len() {
                let chunk = _mm256_loadu_si256(bytes.as_ptr().add(i) as *const __m256i);

                let m_amp = _mm256_cmpeq_epi8(chunk, amp);
                let m_lt = _mm256_cmpeq_epi8(chunk, lt);
                let m_gt = _mm256_cmpeq_epi8(chunk, gt);
                let m_quot = _mm256_cmpeq_epi8(chunk, quot);
                let m_apos = _mm256_cmpeq_epi8(chunk, apos);

                let mask = _mm256_or_si256(
                    _mm256_or_si256(_mm256_or_si256(m_amp, m_lt), _mm256_or_si256(m_gt, m_quot)),
                    m_apos,
                );

                let has_special = _mm256_movemask_epi8(mask);

                if has_special == 0 {
                    result.push_str(std::str::from_utf8_unchecked(&bytes[i..i + 32]));
                    i += 32;
                } else {
                    break;
                }
            }
        }

        for &byte in &bytes[i..] {
            match byte {
                b'&' => result.push_str("&amp;"),
                b'<' => result.push_str("&lt;"),
                b'>' => result.push_str("&gt;"),
                b'"' => result.push_str("&quot;"),
                b'\'' => result.push_str("&apos;"),
                _ => result.push(byte as char),
            }
        }

        result
    }

    #[inline(always)]
    pub fn show_info(&mut self, message: &str) -> Result<()> {
        let title = self.translations.app_title().to_string();
        self.show_notification(&title, message, NIIF_INFO)
    }

    #[inline(always)]
    pub fn show_warning(&mut self, message: &str) -> Result<()> {
        let title = self.translations.app_title().to_string();
        self.show_notification(&title, message, NIIF_WARNING)
    }

    #[inline(always)]
    pub fn show_error(&mut self, message: &str) -> Result<()> {
        let title = self.translations.app_title().to_string();
        self.show_notification(&title, message, NIIF_ERROR)
    }

    pub fn run_message_loop(&mut self) -> Result<()> {
        let state = get_global_state().ok_or(anyhow!("Failed to get app state"))?;

        let (event_tx, event_rx) = crossbeam_channel::unbounded();
        state.set_notification_sender(event_tx);

        let mut msg = MSG::default();
        let mut idle_count = 0u32;
        let mut check_counter = 0u32;

        while !self.should_exit() {
            // Check language update every 10 frames (~100ms)
            check_counter += 1;
            if check_counter >= 10 {
                check_counter = 0;
                self.check_and_update_language();
            }

            while let Ok(event) = event_rx.try_recv() {
                idle_count = 0;

                if state.show_notifications() {
                    match event {
                        NotificationEvent::Info(ref message) => {
                            let _ = self.show_info(message);
                        }
                        NotificationEvent::Warning(ref message) => {
                            let _ = self.show_warning(message);
                        }
                        NotificationEvent::Error(ref message) => {
                            let _ = self.show_error(message);
                        }
                    }
                }
            }

            unsafe {
                let has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();

                if has_message {
                    idle_count = 0;
                    if msg.message == WM_QUIT {
                        break;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    idle_count += 1;
                    let sleep_time = if idle_count < 10 {
                        1
                    } else if idle_count < 50 {
                        5
                    } else {
                        10
                    };
                    Sleep(sleep_time);
                }
            }
        }

        Ok(())
    }

    #[allow(non_snake_case)]
    extern "system" fn window_procedure(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            TRAY_MESSAGE_ID => Self::handle_tray_message(hwnd, lparam),
            WM_DESTROY => Self::handle_destroy(),
            WM_COMMAND => Self::handle_command(wparam),
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    #[allow(non_snake_case)]
    #[inline(always)]
    fn handle_tray_message(hwnd: HWND, lparam: LPARAM) -> LRESULT {
        match lparam.0 as u32 {
            WM_RBUTTONUP => {
                let _ = Self::show_context_menu(hwnd);
            }
            WM_LBUTTONDBLCLK => {
                if let Some(state) = get_global_state() {
                    state.request_show_window();
                }
            }
            _ => {}
        }
        LRESULT(0)
    }

    #[inline(always)]
    fn handle_destroy() -> LRESULT {
        unsafe {
            PostQuitMessage(0);
        }
        LRESULT(0)
    }

    #[inline(always)]
    fn handle_command(wparam: WPARAM) -> LRESULT {
        if let Some(state) = get_global_state() {
            let cmd_id = (wparam.0 as u32 & 0xFFFF) as u16;
            match cmd_id {
                1010 => {
                    let was_paused = state.toggle_paused();
                    if let Some(sender) = state.get_notification_sender() {
                        let language = state.language();
                        let translations = CachedTranslations::new(language);
                        let msg = if was_paused {
                            NotificationEvent::Info(
                                translations.tray_notification_activated().to_string(),
                            )
                        } else {
                            NotificationEvent::Info(
                                translations.tray_notification_paused().to_string(),
                            )
                        };
                        let _ = sender.send(msg);
                    }
                }
                1020 => state.request_show_window(),
                1030 => {
                    state.request_show_window();
                    state.request_show_about();
                }
                1000 => state.exit(),
                _ => {}
            }
        }
        LRESULT(0)
    }

    fn show_context_menu(hwnd: HWND) -> Result<()> {
        let state = get_global_state().ok_or(anyhow!("Failed to get app state"))?;
        let language = state.language();
        let translations = CachedTranslations::new(language);

        unsafe {
            let menu = CreatePopupMenu()?;

            let pause_text: SmallVec<[u16; 64]> = if state.is_paused() {
                translations.tray_activate()
            } else {
                translations.tray_pause()
            }
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

            let show_window_text: SmallVec<[u16; 64]> = translations
                .tray_show_window()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let about_text: SmallVec<[u16; 64]> = translations
                .tray_about()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            let exit_text: SmallVec<[u16; 64]> = translations
                .tray_exit()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();

            AppendMenuW(menu, MF_STRING, 1010, PCWSTR::from_raw(pause_text.as_ptr()))?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            AppendMenuW(
                menu,
                MF_STRING,
                1020,
                PCWSTR::from_raw(show_window_text.as_ptr()),
            )?;
            AppendMenuW(menu, MF_STRING, 1030, PCWSTR::from_raw(about_text.as_ptr()))?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            AppendMenuW(menu, MF_STRING, 1000, PCWSTR::from_raw(exit_text.as_ptr()))?;

            let mut pos = POINT::default();
            GetCursorPos(&mut pos)?;

            let _ = SetForegroundWindow(hwnd);
            let _ = TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_LEFTBUTTON | TPM_BOTTOMALIGN,
                pos.x,
                pos.y,
                Some(0),
                hwnd,
                None,
            );

            let _ = DestroyMenu(menu);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn should_exit(&self) -> bool {
        self.should_exit.load(Ordering::Relaxed)
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
        }
        let _ = Self::unregister_aumid();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape_basic_chars() {
        assert_eq!(TrayIcon::xml_escape_scalar("hello"), "hello");
        assert_eq!(TrayIcon::xml_escape_scalar("test123"), "test123");
    }

    #[test]
    fn test_xml_escape_ampersand() {
        assert_eq!(TrayIcon::xml_escape_scalar("A & B"), "A &amp; B");
        assert_eq!(TrayIcon::xml_escape_scalar("&&&"), "&amp;&amp;&amp;");
    }

    #[test]
    fn test_xml_escape_less_than() {
        assert_eq!(TrayIcon::xml_escape_scalar("A < B"), "A &lt; B");
        assert_eq!(TrayIcon::xml_escape_scalar("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_xml_escape_greater_than() {
        assert_eq!(TrayIcon::xml_escape_scalar("A > B"), "A &gt; B");
    }

    #[test]
    fn test_xml_escape_quotes() {
        assert_eq!(
            TrayIcon::xml_escape_scalar(r#"He said "hi""#),
            "He said &quot;hi&quot;"
        );
        assert_eq!(TrayIcon::xml_escape_scalar("It's ok"), "It&apos;s ok");
    }

    #[test]
    fn test_xml_escape_combined() {
        assert_eq!(
            TrayIcon::xml_escape_scalar(r#"<tag attr="value">&text</tag>"#),
            "&lt;tag attr=&quot;value&quot;&gt;&amp;text&lt;/tag&gt;"
        );
    }

    #[test]
    fn test_xml_escape_empty_string() {
        assert_eq!(TrayIcon::xml_escape_scalar(""), "");
    }

    #[test]
    fn test_utf16_cache() {
        let mut cache = Utf16Cache::new();

        let tooltip = cache.encode_tooltip("Test");
        assert!(tooltip.ends_with(&[0]));
        assert!(tooltip.len() > 1);

        let title = cache.encode_title("Title");
        assert!(title.ends_with(&[0]));

        let msg = cache.encode_message("Message");
        assert!(msg.ends_with(&[0]));
    }
}
