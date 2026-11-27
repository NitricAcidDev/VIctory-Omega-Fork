use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

#[repr(C)]
struct TitleInfo {
    struct_size: u32,
    version: u32,
    image_base: *mut c_void,
    image_size: u64,
    exe_path: *const u16,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(path: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}

/// Start the locally installed ScriptHook VI using its VIctory host ABI.
pub(super) fn start(base: usize, size: u64, exe: &Path) {
    let Ok(exe) = std::fs::canonicalize(exe) else { return };
    let Some(dir) = exe.parent() else { return };
    let dll = dir.join("shvi/shvi_core.dll");
    if !dll.is_file() {
        return;
    }
    let wide = |path: &Path| path.as_os_str().encode_wide().chain(Some(0)).collect::<Vec<u16>>();
    let library = wide(&dll);
    let title_path = wide(&exe);
    const SEARCH_DLL_LOAD_DIR: u32 = 0x100;
    const SEARCH_DEFAULT_DIRS: u32 = 0x1000;
    // SAFETY: the library path is absolute and NUL-terminated. Keep the module
    // loaded because it can register hooks that remain active for this run.
    let module = unsafe {
        LoadLibraryExW(library.as_ptr(), std::ptr::null_mut(), SEARCH_DLL_LOAD_DIR | SEARCH_DEFAULT_DIRS)
    };
    if module.is_null() {
        eprintln!("ScriptHook VI: {}", std::io::Error::last_os_error());
        return;
    }
    // SAFETY: the module is loaded and the export name is NUL-terminated.
    let entry = unsafe { GetProcAddress(module, c"victory_on_title_mapped".as_ptr().cast()) };
    if entry.is_null() {
        eprintln!("ScriptHook VI: hosted entry point is missing");
        return;
    }
    let title = TitleInfo {
        struct_size: size_of::<TitleInfo>() as u32,
        version: 1,
        image_base: base as *mut c_void,
        image_size: size,
        exe_path: title_path.as_ptr(),
    };
    // SAFETY: the export uses the published C ABI. The title stays mapped and
    // ScriptHook VI copies its path during the call.
    let start: unsafe extern "C" fn(*const TitleInfo) -> i32 = unsafe { std::mem::transmute(entry) };
    let status = unsafe { start(&title) };
    eprintln!("ScriptHook VI: hosted start returned {status}");
}
