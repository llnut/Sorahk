use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE,
    Win32::Graphics::Gdi::*,
};

use anyhow::{anyhow, Result};

use std::sync::Once;

static REGISTER_CLASS = Once::new();

// 定义窗口类名和关于信息
const ABOUT_TEXT: &str = "Sorahk v0.1.1\n\nA pure Rust implementation of an AHK turbo function tool for Windows.";

// 安全封装窗口创建和消息循环
pub struct AboutWindow;

impl AboutWindow {
    pub fn show() -> Result<()> {
        unsafe { Self::show_unsafe() }
    }

    unsafe fn show_unsafe() -> Result<()> {
        let instance = unsafe { GetModuleHandleW(None)? };
        let class_name = w!("About Sorahk");

        // 使用Once确保窗口类只注册一次
        REGISTER_CLASS.call_once(|| {
            // 注册窗口类
            let wc = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance.into(),
                hIcon: LoadIconW(None, IDI_APPLICATION)?,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: unsafe { GetSysColorBrush(SYS_COLOR_INDEX(COLOR_WINDOW.0 + 1)) },
                lpszMenuName: PCWSTR::null(),
                lpszClassName: class_name,
            };

            let atom = RegisterClassW(&wc);
            if atom == 0 {
                panic!("Failed to register window class");
                //return Err(anyhow!("Failed to register window class"));
            }
        });

        //// 注册窗口类
        //let wc = WNDCLASSW {
        //    style: CS_HREDRAW | CS_VREDRAW,
        //    lpfnWndProc: Some(Self::window_proc),
        //    cbClsExtra: 0,
        //    cbWndExtra: 0,
        //    hInstance: instance.into(),
        //    hIcon: LoadIconW(None, IDI_APPLICATION)?,
        //    hCursor: LoadCursorW(None, IDC_ARROW)?,
        //    hbrBackground: unsafe { GetSysColorBrush(SYS_COLOR_INDEX(COLOR_WINDOW.0 + 1)) },
        //    lpszMenuName: PCWSTR::null(),
        //    lpszClassName: class_name,
        //};

        //let atom = RegisterClassW(&wc);
        //if atom == 0 {
        //    return Err(anyhow!("Failed to register window class"));
        //}

        // 创建窗口
        //let hwnd = unsafe{ CreateWindowExA(
        //    WINDOW_EX_STYLE::default(),
        //    class_name,
        //    w!("About Sorahk title"),
        //    WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        //    CW_USEDEFAULT,
        //    CW_USEDEFAULT,
        //    400,
        //    300,
        //    None,
        //    None,
        //    Some(instance.into()),
        //    None,
        //)?};
        let hwnd = unsafe{
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("About Sorahk title"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                300,
                140,
                None,
                None,
                Some(instance.into()),
                None,
            )
        }?;

        // 显示窗口
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            UpdateWindow(hwnd);

            // 消息循环
            let mut msg = MSG::default();
            while GetMessageA(&mut msg, Some(HWND::default()), 0, 0).into() {
                let _ = TranslateMessage(&msg);
                DispatchMessageA(&msg);
            }
        }

        Ok(())
    }

    extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_PAINT => {
                    Self::on_paint(hwnd);
                    LRESULT(0)
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_KEYDOWN => {
                    match wparam.0 {
                        0x1B | 0x0D  => {
                            DestroyWindow(hwnd).unwrap();
                        }
                        _ => {}
                    }
                    LRESULT(0)
                }
                //WM_LBUTTONDOWN => {
                //    // 点击关闭
                //    DestroyWindow(hwnd).unwrap();
                //    LRESULT(0)
                //}
                _ => DefWindowProcA(hwnd, msg, wparam, lparam),
            }
        }
    }

    unsafe fn on_paint(hwnd: HWND) {
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut ps);

        // 设置背景色
        let background_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
        FillRect(hdc, &ps.rcPaint, background_brush);
        DeleteObject(HGDIOBJ(background_brush.0));

        // 设置文本颜色和字体
        SetTextColor(hdc, COLORREF(0x00000000));
        SetBkMode(hdc, TRANSPARENT);

        // 选择系统字体
        let hfont = GetStockObject(SYSTEM_FONT);
        SelectObject(hdc, HGDIOBJ(hfont.0));

        // 绘制关于文本
        let mut text: Vec<u16> = ABOUT_TEXT.encode_utf16().collect();
        let rect = RECT {
            left: 20,
            top: 20,
            right: ps.rcPaint.right - 20,
            bottom: ps.rcPaint.bottom - 20,
        };

        DrawTextW(
            hdc,
            text.as_mut(),
            &mut rect.clone(),
            DT_LEFT | DT_TOP | DT_WORDBREAK,
        );

        EndPaint(hwnd, &ps);
    }
}
