use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::System::Threading::Sleep, Win32::UI::Shell::*, Win32::UI::WindowsAndMessaging::*,
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

pub struct TrayIcon {
    nid: NOTIFYICONDATAW,
    should_exit: Arc<AtomicBool>,
}

impl TrayIcon {
    /// Create new tray icon
    pub fn new(should_exit: Arc<AtomicBool>) -> Result<Self> {
        let window_class = w!("SorahkWindowClass");
        let instance = unsafe { GetModuleHandleW(None)? };

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(Self::window_procedure),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: unsafe { LoadIconW::<PCWSTR>(None, IDI_APPLICATION)? },
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

        // Initial tray icon
        let initial_icon = unsafe { LoadIconW::<PCWSTR>(None, IDI_APPLICATION)? };
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

        Ok(Self { nid, should_exit })
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

    /// Show notification
    pub fn show_notification(
        &mut self,
        title: &str,
        message: &str,
        icon_type: NOTIFY_ICON_INFOTIP_FLAGS,
    ) -> Result<()> {
        // Backup old flags
        let original_flags = self.nid.uFlags;

        // Set new flags
        self.nid.uFlags |= NIF_INFO;
        self.nid.dwInfoFlags = icon_type;

        // 添加图标后
        self.nid.Anonymous = NOTIFYICONDATAW_0 {
            uVersion: NOTIFYICON_VERSION,
        };
        if !unsafe { Shell_NotifyIconW(NIM_SETVERSION, &self.nid).into() } {
            return Err(anyhow!("Failed to set notification version"));
        }

        Self::set_notification_text(&mut self.nid, title, message);
        Self::set_notification_timeout(&mut self.nid, 1000);

        if !unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.nid).into() } {
            return Err(anyhow!("Failed to show notification"));
        }

        // Restore flags to backup version
        self.nid.uFlags = original_flags;
        Ok(())
    }

    /// Set the title and message text of the notification
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

    fn set_notification_timeout(nid: &mut NOTIFYICONDATAW, timeout: u32) {
        nid.Anonymous = NOTIFYICONDATAW_0 { uTimeout: timeout };
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

        let show_notifications = state.show_notifications;

        let mut msg = MSG::default();
        while !self.should_exit() {
            if let Ok(event) = event_rx.try_recv() {
                let _ = self.set_icon_by_status();
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

        println!("Exiting the tray message loop...");
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
                if let Err(e) = Self::show_context_menu(hwnd) {
                    eprintln!("Failed to show context menu: {}", e);
                }
            }
            WM_LBUTTONDBLCLK => {
                println!("Tray icon double clicked");
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
                        println!("Sorahk activiting");
                    } else {
                        if let Some(sender) = state.get_notification_sender() {
                            let _ =
                                sender.send(NotificationEvent::Info("Sorahk paused".to_string()));
                        }
                        println!("Sorahk paused");
                    }
                }
                //1001 => {
                //    if let Err(e) = AboutWindow::show() {
                //        eprintln!("创建关于窗口失败: {:?}", e);
                //    }
                //}
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
                    w!("Activate Sorahk")
                } else {
                    w!("Pause Sorahk")
                },
            )?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            //AppendMenuW(menu, MF_STRING, 1001, w!("About Sorahk"))?;
            AppendMenuW(menu, MF_STRING, 1000, w!("Exit"))?;

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
    /// Automatically clean the tray icon
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
            // Note: We do not destroy system icons; we only destroy custom-created icons
            // In practical applications, if it is necessary to destroy custom icons, it should be handled here
        }
    }
}
