use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};
use windows_sys::Win32::UI::{Input::KeyboardAndMouse::VK_RETURN, WindowsAndMessaging as wm};

/// The game's desktop window name.
pub const TITLE: &str = "Victory | Omega";

/// Set the caption and icons of the top-level window on its owning thread.
pub fn brand(hwnd: HWND) {
    const PNG: &[u8] = include_bytes!("../assets/omega.png");
    #[repr(align(4))]
    struct Aligned([u8; PNG.len()]);
    static IMAGE: Aligned = Aligned(*include_bytes!("../assets/omega.png"));
    static ICONS: OnceLock<[usize; 2]> = OnceLock::new();
    // SAFETY: a live window on its owner thread and a static, aligned PNG.
    unsafe {
        if wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32 & wm::WS_CHILD != 0 {
            return;
        }
        let title: Vec<u16> = TITLE.encode_utf16().chain([0]).collect();
        wm::SetWindowTextW(hwnd, title.as_ptr());
        // The same handles are shared by the windows for the process lifetime.
        let icons = ICONS.get_or_init(|| {
            [
                (wm::SM_CXSMICON, wm::SM_CYSMICON),
                (wm::SM_CXICON, wm::SM_CYICON),
            ]
            .map(|(width, height)| {
                wm::CreateIconFromResourceEx(
                    IMAGE.0.as_ptr(),
                    PNG.len() as u32,
                    1,
                    0x0003_0000,
                    wm::GetSystemMetrics(width),
                    wm::GetSystemMetrics(height),
                    0,
                ) as usize
            })
        });
        for (size, icon) in [wm::ICON_SMALL, wm::ICON_BIG].into_iter().zip(icons) {
            if *icon != 0 {
                wm::SendMessageW(hwnd, wm::WM_SETICON, size as usize, *icon as isize);
            }
        }
    }
}

struct Windowed {
    style: u32,
    placement: wm::WINDOWPLACEMENT,
}

thread_local! {
    static WINDOWED: RefCell<HashMap<usize, Windowed>> = RefCell::new(HashMap::new());
}

/// Handle Alt+Enter on the window's own thread. Borderless mode keeps the
/// swap chain windowed while Windows scales the picture to the monitor.
pub fn handle_message(hwnd: HWND, message: u32, key: WPARAM, flags: LPARAM) -> bool {
    if message == wm::WM_NCDESTROY {
        WINDOWED.with(|windows| windows.borrow_mut().remove(&(hwnd as usize)));
    }
    if message != wm::WM_SYSKEYDOWN || key != usize::from(VK_RETURN) || flags & (1 << 29) == 0 {
        return false;
    }
    if flags & (1 << 30) == 0 {
        toggle(hwnd);
    }
    true
}

fn toggle(hwnd: HWND) {
    let saved = WINDOWED.with(|windows| windows.borrow_mut().remove(&(hwnd as usize)));
    // SAFETY: the caller is handling a message for this window and all
    // placement and monitor information is read into local structures.
    unsafe {
        let style = wm::GetWindowLongW(hwnd, wm::GWL_STYLE) as u32;
        if style & wm::WS_CHILD != 0 {
            return;
        }
        if let Some(saved) = saved {
            wm::SetWindowLongW(hwnd, wm::GWL_STYLE, saved.style as i32);
            wm::SetWindowPlacement(hwnd, &saved.placement);
            wm::SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                wm::SWP_FRAMECHANGED
                    | wm::SWP_NOZORDER
                    | wm::SWP_NOACTIVATE
                    | wm::SWP_NOMOVE
                    | wm::SWP_NOSIZE,
            );
            return;
        }

        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..std::mem::zeroed()
        };
        let mut placement = wm::WINDOWPLACEMENT {
            length: size_of::<wm::WINDOWPLACEMENT>() as u32,
            ..std::mem::zeroed()
        };
        if GetMonitorInfoW(monitor, &mut info) == 0
            || wm::GetWindowPlacement(hwnd, &mut placement) == 0
        {
            return;
        }
        WINDOWED.with(|windows| {
            windows
                .borrow_mut()
                .insert(hwnd as usize, Windowed { style, placement })
        });
        // Keep the taskbar's minimize/restore command available in borderless mode.
        let frame = wm::WS_CAPTION | wm::WS_THICKFRAME | wm::WS_MAXIMIZEBOX | wm::WS_MAXIMIZE;
        let borderless = style & !frame | wm::WS_POPUP;
        wm::SetWindowLongW(hwnd, wm::GWL_STYLE, borderless as i32);
        let area = info.rcMonitor;
        wm::SetWindowPos(
            hwnd,
            std::ptr::null_mut(),
            area.left,
            area.top,
            area.right - area.left,
            area.bottom - area.top,
            wm::SWP_FRAMECHANGED | wm::SWP_NOZORDER | wm::SWP_NOACTIVATE,
        );
    }
}
