use windows::Win32::System::Console::*;
use windows::core::*;

use crate::state::get_global_state;

pub fn set_control_ctrl_handler() -> Result<()> {
    unsafe { SetConsoleCtrlHandler(Some(console_handler), true) }
}

#[allow(non_snake_case)]
unsafe extern "system" fn console_handler(ctrl_type: u32) -> BOOL {
    match ctrl_type {
        CTRL_C_EVENT | CTRL_BREAK_EVENT | CTRL_CLOSE_EVENT => {
            match get_global_state() {
                Some(state) => state.exit(),   // graceful shutdown
                None => std::process::exit(0), // force shutdown
            }
            BOOL(1) // Event has been handled
        }
        _ => BOOL(0), // Leave other events to the default handler
    }
}
