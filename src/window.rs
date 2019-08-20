extern crate winapi;

use std::{os, vec::Vec};
use winapi::shared::{minwindef, ntdef, windef};
use winapi::um::{libloaderapi, shellscalingapi, wingdi, winuser};

type SetProcessDPIAware = unsafe extern "system" fn() -> minwindef::BOOL;
type SetProcessDpiAwareness =
    unsafe extern "system" fn(value: shellscalingapi::PROCESS_DPI_AWARENESS) -> ntdef::HRESULT;
type SetProcessDpiAwarenessContext =
    unsafe extern "system" fn(value: windef::DPI_AWARENESS_CONTEXT) -> minwindef::BOOL;
type GetDpiForWindow = unsafe extern "system" fn(hwnd: windef::HWND) -> minwindef::UINT;
type GetDpiForMonitor = unsafe extern "system" fn(
    hmonitor: windef::HMONITOR,
    dpi_type: shellscalingapi::MONITOR_DPI_TYPE,
    dpi_x: *mut minwindef::UINT,
    dpi_y: *mut minwindef::UINT,
) -> ntdef::HRESULT;
type EnableNonClientDpiScaling = unsafe extern "system" fn(hwnd: windef::HWND) -> minwindef::BOOL;

// Helper function to dynamically load function pointer.
// `library` and `function` must be zero-terminated.
fn get_function_impl(library: &str, function: &str) -> Option<*const os::raw::c_void> {
    // Library names we will use are ASCII so we can use the A version to avoid string conversion.
    let module =
        unsafe { libloaderapi::LoadLibraryA(library.as_ptr() as winapi::um::winnt::LPCSTR) };
    if module.is_null() {
        return None;
    }

    let function_ptr = unsafe {
        libloaderapi::GetProcAddress(module, function.as_ptr() as winapi::um::winnt::LPCSTR)
    };
    if function_ptr.is_null() {
        return None;
    }

    Some(function_ptr as _)
}

macro_rules! get_function {
    ( $ lib: expr, $ func: ident) => {
        get_function_impl(concat!($lib, '\0'), concat!(stringify!($func), '\0'))
            .map(|f| unsafe { std::mem::transmute::<*const _, $func>(f) })
    };
}

pub struct DpiFunctions {
    get_dpi_for_window: Option<GetDpiForWindow>,
    get_dpi_for_monitor: Option<GetDpiForMonitor>,
    enable_nonclient_dpi_scaling: Option<EnableNonClientDpiScaling>,
    _set_process_dpi_awareness_context: Option<SetProcessDpiAwarenessContext>,
    _set_process_dpi_awareness: Option<SetProcessDpiAwareness>,
    _set_process_dpi_aware: Option<SetProcessDPIAware>,
}

const BASE_DPI: u32 = 96;

impl DpiFunctions {
    fn new() -> DpiFunctions {
        DpiFunctions {
            get_dpi_for_window: get_function!("user32.dll", GetDpiForWindow),
            get_dpi_for_monitor: get_function!("shcore.dll", GetDpiForMonitor),
            enable_nonclient_dpi_scaling: get_function!("user32.dll", EnableNonClientDpiScaling),
            _set_process_dpi_awareness_context: get_function!(
                "user32.dll",
                SetProcessDpiAwarenessContext
            ),
            _set_process_dpi_awareness: get_function!("shcore.dll", SetProcessDpiAwareness),
            _set_process_dpi_aware: get_function!("user32.dll", SetProcessDPIAware),
        }
    }

    pub fn enable_non_client_dpi_scaling(&self, hwnd: windef::HWND) {
        unsafe {
            if let Some(enable_nonclient_dpi_scaling) = self.enable_nonclient_dpi_scaling {
                enable_nonclient_dpi_scaling(hwnd);
            }
        }
    }
    /*
    pub fn get_monitor_dpi(hmonitor: HMONITOR) -> Option<u32> {
        unsafe {
            if let Some(GetDpiForMonitor) = *GET_DPI_FOR_MONITOR {
                // We are on Windows 8.1 or later.
                let mut dpi_x = 0;
                let mut dpi_y = 0;
                if GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) == S_OK {
                    // MSDN says that "the values of *dpiX and *dpiY are identical. You only need to
                    // record one of the values to determine the DPI and respond appropriately".
                    // https://msdn.microsoft.com/en-us/library/windows/desktop/dn280510(v=vs.85).aspx
                    return Some(dpi_x as u32)
                }
            }
        }
        None
    }*/

    pub fn hwnd_dpi_factor(&self, hwnd: windef::HWND) -> f32 {
        unsafe {
            let hdc = winuser::GetDC(hwnd);
            if hdc.is_null() {
                panic!("`GetDC` returned null!");
            }
            let dpi = if let Some(get_dpi_for_window) = self.get_dpi_for_window {
                // We are on Windows 10 Anniversary Update (1607) or later.
                match get_dpi_for_window(hwnd) {
                    0 => BASE_DPI, // 0 is returned if hwnd is invalid
                    dpi => dpi as u32,
                }
            } else if let Some(get_dpi_for_monitor) = self.get_dpi_for_monitor {
                // We are on Windows 8.1 or later.
                let monitor = winuser::MonitorFromWindow(hwnd, winuser::MONITOR_DEFAULTTONEAREST);
                if monitor.is_null() {
                    BASE_DPI
                } else {
                    let mut dpi_x = 0;
                    let mut dpi_y = 0;
                    if get_dpi_for_monitor(
                        monitor,
                        shellscalingapi::MDT_EFFECTIVE_DPI,
                        &mut dpi_x,
                        &mut dpi_y,
                    ) == winapi::shared::winerror::S_OK
                    {
                        dpi_x as u32
                    } else {
                        BASE_DPI
                    }
                }
            } else {
                // We are on Vista or later.
                if winuser::IsProcessDPIAware() != minwindef::FALSE {
                    // If the process is DPI aware, then scaling must be handled by the application using
                    // this DPI value.
                    wingdi::GetDeviceCaps(hdc, wingdi::LOGPIXELSX) as u32
                } else {
                    // If the process is DPI unaware, then scaling is performed by the OS; we thus return
                    // 96 (scale factor 1.0) to prevent the window from being re-scaled by both the
                    // application and the WM.
                    BASE_DPI
                }
            };
            dpi as f32 / BASE_DPI as f32
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Point2DF32 {
    x: f32,
    y: f32,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct WindowGeom {
    pub dpi_factor: f32,
    pub is_fullscreen: bool,
    pub is_topmost: bool,
    pub position: Point2DF32,
    pub inner_size: Point2DF32,
    pub outer_size: Point2DF32,
}

pub struct Window {
    pub hwnd: *mut windef::HWND__,
    dpi_functions: DpiFunctions,
}

impl Window {
    pub unsafe fn new(name: Vec<u16>, title: Vec<u16>) -> Window {
        let hinstance = libloaderapi::GetModuleHandleW(std::ptr::null());

        let class = winuser::WNDCLASSW {
            style: winuser::CS_HREDRAW | winuser::CS_VREDRAW | winuser::CS_OWNDC,
            lpfnWndProc: Some(winuser::DefWindowProcW),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: winuser::LoadIconW(std::ptr::null_mut(), winuser::IDI_WINLOGO),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: name.as_ptr() as *const _,
        };

        winuser::RegisterClassW(&class);
        winuser::IsGUIThread(1);

        let hwnd = winuser::CreateWindowExW(
            0,
            name.as_ptr(),
            title.as_ptr(),
            winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );

        if hwnd.is_null() {
            panic!("could not create hwnd");
        }

        Window {
            hwnd,
            dpi_functions: DpiFunctions::new(),
        }
    }

    pub fn get_is_topmost(&self) -> bool {
        unsafe {
            let ex_style = winuser::GetWindowLongW(self.hwnd, winuser::GWL_EXSTYLE) as u32;
            if (ex_style & winuser::WS_EX_TOPMOST) != 0 {
                return true;
            }
            return false;
        }
    }

    pub fn get_is_maximized(&self) -> bool {
        unsafe {
            let mut wp: winuser::WINDOWPLACEMENT = std::mem::zeroed();
            wp.length = std::mem::size_of::<winuser::WINDOWPLACEMENT>() as u32;
            winuser::GetWindowPlacement(self.hwnd, &mut wp);
            if wp.showCmd as i32 == winuser::SW_MAXIMIZE {
                return true;
            }
            return false;
        }
    }

    pub fn get_position(&self) -> Point2DF32 {
        unsafe {
            let mut rect = windef::RECT {
                left: 0,
                top: 0,
                bottom: 0,
                right: 0,
            };
            winuser::GetWindowRect(self.hwnd, &mut rect);
            Point2DF32 {
                x: rect.left as f32,
                y: rect.top as f32,
            }
        }
    }

    pub fn get_inner_size(&self) -> Point2DF32 {
        unsafe {
            let mut rect = windef::RECT {
                left: 0,
                top: 0,
                bottom: 0,
                right: 0,
            };
            winuser::GetClientRect(self.hwnd, &mut rect);
            let dpi = self.get_dpi_factor();
            Point2DF32 {
                x: (rect.right - rect.left) as f32 / dpi,
                y: (rect.bottom - rect.top) as f32 / dpi,
            }
        }
    }

    pub fn get_outer_size(&self) -> Point2DF32 {
        unsafe {
            let mut rect = windef::RECT {
                left: 0,
                top: 0,
                bottom: 0,
                right: 0,
            };
            winuser::GetWindowRect(self.hwnd, &mut rect);
            Point2DF32 {
                x: (rect.right - rect.left) as f32,
                y: (rect.bottom - rect.top) as f32,
            }
        }
    }

    pub unsafe fn get_width(&self) -> u32 {
        self.get_inner_size().x as u32
    }

    pub unsafe fn get_height(&self) -> u32 {
        self.get_inner_size().y as u32
    }

    pub fn get_dpi_factor(&self) -> f32 {
        (*self).dpi_functions.hwnd_dpi_factor(self.hwnd)
    }

    pub fn get_window_geom(&self) -> WindowGeom {
        WindowGeom {
            is_topmost: self.get_is_topmost(),
            is_fullscreen: self.get_is_maximized(),
            inner_size: self.get_inner_size(),
            outer_size: self.get_outer_size(),
            dpi_factor: self.get_dpi_factor(),
            position: self.get_position(),
        }
    }
}

pub unsafe fn quit(wnd: &mut Window) -> bool {
    let mut message: winuser::MSG = std::mem::zeroed();

    if winuser::GetMessageW(&mut message as *mut winuser::MSG, wnd.hwnd, 0, 0) == 0 {
        true
    } else {
        false
    }
}
