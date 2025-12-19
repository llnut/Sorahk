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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
//use std::thread;

use crate::state::{NotificationEvent, get_global_state};
//use crate::about::AboutWindow;

const TRAY_MESSAGE_ID: u32 = WM_APP + 1;
const AUMID: &str = "Sorahk.AutoKeyPress";

pub struct TrayIcon {
    nid: NOTIFYICONDATAW,
    should_exit: Arc<AtomicBool>,
}

impl TrayIcon {
    /// Create new tray icon
    pub fn new(should_exit: Arc<AtomicBool>) -> Result<Self> {
        let window_class = w!("SorahkWindowClass");
        let instance = unsafe { GetModuleHandleW(None)? };

        // Try to load embedded icon for window class
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

        // Create a hidden message window
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

        // Load embedded icon (ID: 1) or fallback to system icon
        let initial_icon = unsafe {
            // Try to load embedded icon from resources
            #[allow(clippy::manual_dangling_ptr)]
            let embedded_icon = LoadIconW(Some(instance.into()), PCWSTR::from_raw(1 as *const u16));

            // If embedded icon fails, use system default
            if let Ok(icon) = embedded_icon {
                icon
            } else {
                LoadIconW::<PCWSTR>(None, IDI_APPLICATION)?
            }
        };
        // Set the tray icon data
        let mut nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: WM_APP + 1,
            hIcon: initial_icon,
            ..Default::default()
        };

        // Set the tooltip text
        Self::set_tooltip(&mut nid, "Sorahk");

        // Add the tray icon
        let _ = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };

        // Register AUMID for Toast notifications
        let _ = Self::register_aumid();

        Ok(Self { nid, should_exit })
    }

    /// Register AUMID in the registry for Toast notifications
    fn register_aumid() -> Result<()> {
        unsafe {
            // Registry path: HKEY_CURRENT_USER\Software\Classes\AppUserModelId\{AUMID}
            let registry_path = format!("Software\\Classes\\AppUserModelId\\{}", AUMID);
            let registry_path_wide: Vec<u16> = registry_path
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
                return Err(anyhow!("Failed to create registry key: {:?}", result));
            }

            // Set DisplayName
            let display_name = "Sorahk - Auto Keypress Tool";
            let display_name_wide: Vec<u16> = display_name
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            let display_name_bytes = std::slice::from_raw_parts(
                display_name_wide.as_ptr() as *const u8,
                display_name_wide.len() * 2,
            );

            let result = RegSetValueExW(
                hkey,
                w!("DisplayName"),
                Some(0),
                REG_SZ,
                Some(display_name_bytes),
            );

            if result.is_err() {
                let _ = RegCloseKey(hkey);
                return Err(anyhow!("Failed to set DisplayName: {:?}", result));
            }

            // Set IconUri to the executable path (Windows will extract icon from exe)
            if let Ok(exe_path) = std::env::current_exe() {
                let icon_uri = exe_path.to_string_lossy().to_string();
                let icon_uri_wide: Vec<u16> =
                    icon_uri.encode_utf16().chain(std::iter::once(0)).collect();
                let icon_uri_bytes = std::slice::from_raw_parts(
                    icon_uri_wide.as_ptr() as *const u8,
                    icon_uri_wide.len() * 2,
                );

                let result =
                    RegSetValueExW(hkey, w!("IconUri"), Some(0), REG_SZ, Some(icon_uri_bytes));

                // Continue anyway, DisplayName is more important
                let _ = result;
            }

            // Close registry key
            let _ = RegCloseKey(hkey);

            Ok(())
        }
    }

    /// Unregister AUMID from registry (cleanup)
    fn unregister_aumid() -> Result<()> {
        unsafe {
            let registry_path = format!("Software\\Classes\\AppUserModelId\\{}", AUMID);
            let registry_path_wide: Vec<u16> = registry_path
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
                Err(anyhow!("Failed to unregister AUMID: {:?}", result))
            }
        }
    }

    /// Set new tray icon
    #[allow(unused)]
    pub fn set_icon(&mut self, icon: HICON) -> Result<()> {
        self.nid.hIcon = icon;

        // Make sure the icon flags is set
        self.nid.uFlags |= NIF_ICON;
        // Update tray icon
        if !unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.nid).into() } {
            return Err(anyhow!("Failed to set icon"));
        }
        Ok(())
    }

    /// Set the tray icon by using system predefined icons
    #[allow(unused)]
    pub fn set_system_icon(&mut self, icon_id: PCWSTR) -> Result<()> {
        let icon = unsafe { LoadIconW(None, icon_id)? };
        self.set_icon(icon)
    }

    /// Load icon from file (supports ICO files)
    #[allow(unused)]
    pub fn set_icon_from_file(&mut self, file_path: &str) -> Result<()> {
        let wide_path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();
        let path = PCWSTR::from_raw(wide_path.as_ptr());
        let icon = unsafe {
            LoadImageW(
                None,
                path,
                IMAGE_ICON,
                0, // default size
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            )?
        };
        self.set_icon(HICON(icon.0))
    }

    /// Load icon from app resources
    #[allow(unused)]
    pub fn set_icon_from_resource(
        &mut self,
        instance: HINSTANCE,
        resource_name: PCWSTR,
    ) -> Result<()> {
        let icon = unsafe {
            LoadImageW(
                Some(instance),
                resource_name,
                IMAGE_ICON,
                0,
                0,
                LR_DEFAULTSIZE,
            )?
        };
        self.set_icon(HICON(icon.0))
    }

    /// Create simple color icons (for dynamically generating icons)
    #[allow(unused)]
    pub fn set_color_icon(&mut self, red: u8, green: u8, blue: u8) -> Result<()> {
        let icon = self.create_solid_color_icon(red, green, blue)?;
        self.set_icon(icon)
    }

    /// Create a monochrome icon
    fn create_solid_color_icon(&self, red: u8, green: u8, blue: u8) -> Result<HICON> {
        // Here, the implementation is simplified. In practical applications, more complex icon creation logic may be required
        // Use system-predefined icons as examples. In practical applications, custom icons can be created using GDI+ or other libraries
        // Select different system icons based on color (simplified implementation)
        let icon_id = match (red, green, blue) {
            (255, 0, 0) => IDI_ERROR,     // red -> error icon
            (255, 255, 0) => IDI_WARNING, // yellow -> warning icon
            (0, 255, 0) => IDI_ASTERISK,  // green -> info icon
            _ => IDI_APPLICATION,         // default icon
        };

        unsafe { Ok(LoadIconW(None, icon_id)?) }
    }

    /// Set tooltip text for tray icon
    fn set_tooltip(nid: &mut NOTIFYICONDATAW, tooltip: &str) {
        let tip_wide: Vec<u16> = tooltip.encode_utf16().chain(std::iter::once(0)).collect();
        let copy_len = tip_wide.len().min(nid.szTip.len());
        nid.szTip[..copy_len].copy_from_slice(&tip_wide[..copy_len]);
    }

    /// Show notification using modern Windows Toast Notifications (Windows 10+)
    /// This supports multiple notifications and automatic updates
    pub fn show_notification(
        &mut self,
        title: &str,
        message: &str,
        icon_type: NOTIFY_ICON_INFOTIP_FLAGS,
    ) -> Result<()> {
        // Try Toast notification with registered AUMID
        match Self::show_toast_notification(title, message) {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback to legacy Shell_NotifyIcon if Toast fails
                self.show_legacy_notification(title, message, icon_type)
            }
        }
    }

    /// Modern Toast Notification implementation (Windows 10+)
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
            Self::xml_escape(title),
            Self::xml_escape(message)
        );

        // Use the registered AUMID
        Self::try_show_toast_with_aumid(&toast_xml, AUMID)
    }

    /// Try to show toast with a specific AUMID
    fn try_show_toast_with_aumid(toast_xml: &str, app_id: &str) -> Result<()> {
        // Create XML document
        let xml_doc = XmlDocument::new()?;
        xml_doc.LoadXml(&HSTRING::from(toast_xml))?;

        // Create toast notification
        let toast = ToastNotification::CreateToastNotification(&xml_doc)?;

        // Get toast notifier with specific AUMID
        let aumid = HSTRING::from(app_id);
        let notifier = ToastNotificationManager::CreateToastNotifierWithId(&aumid)?;

        // Show the toast
        notifier.Show(&toast)?;

        Ok(())
    }

    /// Fallback: Legacy notification using Shell_NotifyIcon
    fn show_legacy_notification(
        &mut self,
        title: &str,
        message: &str,
        icon_type: NOTIFY_ICON_INFOTIP_FLAGS,
    ) -> Result<()> {
        let original_flags = self.nid.uFlags;

        // Set notification flags and info
        self.nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP | NIF_INFO;
        self.nid.dwInfoFlags = icon_type | NIIF_NOSOUND; // Use NIIF_NOSOUND instead of RESPECT_QUIET_TIME

        Self::set_notification_text(&mut self.nid, title, message);
        self.nid.Anonymous = NOTIFYICONDATAW_0 { uTimeout: 5000 }; // 5 seconds

        let result = unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.nid) };

        if !result.as_bool() {
            self.nid.uFlags = original_flags;
            return Err(anyhow!("Failed to show legacy notification"));
        }

        self.nid.uFlags = original_flags;
        Ok(())
    }

    /// Escape XML special characters
    fn xml_escape(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }

    /// Set the title and message text of the notification (for legacy notifications)
    fn set_notification_text(nid: &mut NOTIFYICONDATAW, title: &str, message: &str) {
        // Set title
        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let title_len = title_wide.len().min(nid.szInfoTitle.len());
        nid.szInfoTitle[..title_len].copy_from_slice(&title_wide[..title_len]);

        // Set message
        let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        let message_len = message_wide.len().min(nid.szInfo.len());
        nid.szInfo[..message_len].copy_from_slice(&message_wide[..message_len]);
    }

    /// Show notification of a info level
    #[allow(unused)]
    pub fn show_info(&mut self, title: &str, message: &str) -> Result<()> {
        self.show_notification(title, message, NIIF_INFO)
    }

    /// Show notification of a warning level
    #[allow(unused)]
    pub fn show_warning(&mut self, title: &str, message: &str) -> Result<()> {
        self.show_notification(title, message, NIIF_WARNING)
    }

    /// Show notification of a error level
    #[allow(unused)]
    pub fn show_error(&mut self, title: &str, message: &str) -> Result<()> {
        self.show_notification(title, message, NIIF_ERROR)
    }

    /// Run notification message loop
    pub fn run_message_loop(&mut self) -> Result<()> {
        let state = get_global_state().ok_or(anyhow!("Failed to get app state"))?;
        let (event_tx, event_rx) = channel();
        state.set_notification_sender(event_tx);

        let mut msg = MSG::default();
        while !self.should_exit() {
            if let Ok(event) = event_rx.try_recv() {
                // Check show_notifications dynamically (in case user changed it in settings)
                let show_notifications = state.show_notifications();

                if show_notifications {
                    match event {
                        NotificationEvent::Info(message) => {
                            let _ = self.show_info("Sorahk", &message);
                        }
                        NotificationEvent::Warning(message) => {
                            let _ = self.show_warning("Sorahk", &message);
                        }
                        NotificationEvent::Error(message) => {
                            let _ = self.show_error("Sorahk", &message);
                        }
                    }
                }
            }

            unsafe {
                let has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();

                if has_message {
                    if msg.message == WM_QUIT {
                        break;
                    }
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    // Take a short sleep when there is no message to avoid high CPU usage
                    Sleep(10);
                }
            }
        }

        Ok(())
    }

    /// Window procedure
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

    /// Handle tray icon message
    #[allow(non_snake_case)]
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

    /// Handle window destory message
    fn handle_destroy() -> LRESULT {
        unsafe {
            PostQuitMessage(0);
        }
        LRESULT(0)
    }

    /// Handle menu command
    fn handle_command(wparam: WPARAM) -> LRESULT {
        if let Some(state) = get_global_state() {
            let cmd_id = Self::loword(wparam.0 as u32);
            match cmd_id {
                1010 => {
                    let was_paused = state.toggle_paused();
                    if was_paused {
                        if let Some(sender) = state.get_notification_sender() {
                            let _ = sender
                                .send(NotificationEvent::Info("Sorahk activiting".to_string()));
                        }
                    } else if let Some(sender) = state.get_notification_sender() {
                        let _ = sender.send(NotificationEvent::Info("Sorahk paused".to_string()));
                    }
                }
                1020 => {
                    // Show Window
                    state.request_show_window();
                }
                1030 => {
                    // Show About dialog
                    state.request_show_window(); // First show the window
                    state.request_show_about(); // Then show about dialog
                }
                1000 => {
                    state.exit();
                }
                _ => {}
            }
        }
        LRESULT(0)
    }

    /// Show context menu
    fn show_context_menu(hwnd: HWND) -> Result<()> {
        let state = get_global_state().ok_or(anyhow!("Failed to get app state"))?;
        unsafe {
            let menu = CreatePopupMenu()?;

            AppendMenuW(
                menu,
                MF_STRING,
                1010,
                if state.is_paused() {
                    w!("✓ Activate Sorahk")
                } else {
                    w!("⏸ Pause Sorahk")
                },
            )?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            AppendMenuW(menu, MF_STRING, 1020, w!("✨️ Show Window"))?;
            AppendMenuW(menu, MF_STRING, 1030, w!("❤ About"))?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            AppendMenuW(menu, MF_STRING, 1000, w!("🚪 Exit Program"))?;

            // Get the mouse position
            let mut pos = POINT::default();
            GetCursorPos(&mut pos)?;

            // Set window to front desk and display the menu
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

    /// Extract the lower 16 bits of the 32-bit value
    fn loword(value: u32) -> u16 {
        (value & 0xFFFF) as u16
    }

    /// Set the tray icon according to the application status
    #[allow(unused)]
    pub fn set_icon_by_status(&mut self) -> Result<()> {
        let icon_id = match get_global_state()
            .ok_or(anyhow!("Failed to get global state"))?
            .is_paused()
        {
            true => IDI_INFORMATION,
            false => IDI_APPLICATION,
        };
        self.set_system_icon(icon_id)
    }

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
        assert_eq!(TrayIcon::xml_escape("hello"), "hello");
        assert_eq!(TrayIcon::xml_escape("test123"), "test123");
    }

    #[test]
    fn test_xml_escape_ampersand() {
        assert_eq!(TrayIcon::xml_escape("A & B"), "A &amp; B");
        assert_eq!(TrayIcon::xml_escape("&&&"), "&amp;&amp;&amp;");
    }

    #[test]
    fn test_xml_escape_less_than() {
        assert_eq!(TrayIcon::xml_escape("A < B"), "A &lt; B");
        assert_eq!(TrayIcon::xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn test_xml_escape_greater_than() {
        assert_eq!(TrayIcon::xml_escape("A > B"), "A &gt; B");
    }

    #[test]
    fn test_xml_escape_quotes() {
        assert_eq!(
            TrayIcon::xml_escape(r#"He said "hi""#),
            "He said &quot;hi&quot;"
        );
        assert_eq!(TrayIcon::xml_escape("It's ok"), "It&apos;s ok");
    }

    #[test]
    fn test_xml_escape_combined() {
        assert_eq!(
            TrayIcon::xml_escape(r#"<tag attr="value">&text</tag>"#),
            "&lt;tag attr=&quot;value&quot;&gt;&amp;text&lt;/tag&gt;"
        );
    }

    #[test]
    fn test_xml_escape_empty_string() {
        assert_eq!(TrayIcon::xml_escape(""), "");
    }

    #[test]
    fn test_loword_extraction() {
        assert_eq!(TrayIcon::loword(0x12345678), 0x5678);
        assert_eq!(TrayIcon::loword(0xABCDEF01), 0xEF01);
        assert_eq!(TrayIcon::loword(0x0000FFFF), 0xFFFF);
        assert_eq!(TrayIcon::loword(0x12340000), 0x0000);
    }

    #[test]
    fn test_notification_text_formatting() {
        // Test that notification text can be properly formatted
        let title = "Test Title";
        let message = "Test Message";

        // XML escape should handle special characters
        let escaped_title = TrayIcon::xml_escape(title);
        let escaped_message = TrayIcon::xml_escape(message);

        assert_eq!(escaped_title, "Test Title");
        assert_eq!(escaped_message, "Test Message");
    }

    #[test]
    fn test_notification_with_special_chars() {
        let title = "Error: Failed <important>";
        let message = "Details: 5 > 3 & 2 < 4";

        let escaped_title = TrayIcon::xml_escape(title);
        let escaped_message = TrayIcon::xml_escape(message);

        assert!(escaped_title.contains("&lt;"));
        assert!(escaped_title.contains("&gt;"));
        assert!(escaped_message.contains("&amp;"));
        assert!(escaped_message.contains("&lt;"));
        assert!(escaped_message.contains("&gt;"));
    }

    #[test]
    fn test_aumid_constant() {
        assert_eq!(AUMID, "Sorahk.AutoKeyPress");
        assert!(!AUMID.is_empty());
    }

    #[test]
    fn test_tray_message_id() {
        assert!(TRAY_MESSAGE_ID > WM_APP);
        assert_eq!(TRAY_MESSAGE_ID, WM_APP + 1);
    }
}
