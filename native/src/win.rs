use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    ffi::c_void,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    Storage::FileSystem::*,
    System::{
        Com::*, DataExchange::*, LibraryLoader::*, Registry::*, SystemInformation::*, Threading::*,
    },
    UI::{Input::KeyboardAndMouse::*, Shell::*, WindowsAndMessaging::*},
};
const CLASS: &str = "AutoSnips.OnDemand.0.2";
const UPDATE: u32 = 0x031d;
const CANCEL: u32 = WM_APP + 1;
const MAX_BYTES: usize = 100 * 1024 * 1024;
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
struct Settings {
    save_folder: String,
    diagnostics: bool,
    broker_candidates: bool,
    launch_method: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            save_folder: String::new(),
            diagnostics: true,
            broker_candidates: true,
            launch_method: "shortcut".into(),
        }
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn root() -> Result<PathBuf, String> {
    let p = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("Local app data is unavailable")?)
        .join("AutoSnipsBrowser");
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    Ok(p)
}
fn settings() -> Result<Settings, String> {
    let p = root()?.join("settings.json");
    if p.exists() {
        serde_json::from_str(&fs::read_to_string(p).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Invalid settings: {e}"))
    } else {
        let old = PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
            .join("AutoSnips/settings.json");
        let s = fs::read_to_string(old)
            .ok()
            .and_then(|v| serde_json::from_str(&v).ok())
            .unwrap_or_default();
        save_settings(&s)?;
        Ok(s)
    }
}
fn save_settings(s: &Settings) -> Result<(), String> {
    let r = root()?;
    let temp = r.join(format!("settings-{}.tmp", std::process::id()));
    fs::write(&temp, serde_json::to_vec_pretty(s).unwrap()).map_err(|e| e.to_string())?;
    unsafe {
        if MoveFileExW(
            wide(&temp.to_string_lossy()).as_ptr(),
            wide(&r.join("settings.json").to_string_lossy()).as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        ) == 0
        {
            let _ = fs::remove_file(temp);
            return Err("Could not update settings".into());
        }
    }
    Ok(())
}
fn folder(s: &Settings) -> Result<PathBuf, String> {
    if !s.save_folder.is_empty() {
        let p = PathBuf::from(&s.save_folder);
        if !p.is_absolute() {
            return Err("Choose an absolute folder path, such as D:\\Pictures\\Screenshots".into());
        }
        return Ok(p);
    }
    unsafe {
        let mut p = std::ptr::null_mut();
        if SHGetKnownFolderPath(&FOLDERID_Screenshots, 0, std::ptr::null_mut(), &mut p) < 0 {
            return Err(
                "Windows' Screenshots folder is unavailable; choose a folder in Settings".into(),
            );
        }
        let mut n = 0;
        while *p.add(n) != 0 {
            n += 1;
        }
        let v = PathBuf::from(String::from_utf16_lossy(std::slice::from_raw_parts(p, n)));
        CoTaskMemFree(p as *const c_void);
        Ok(v)
    }
}
fn check_folder(p: &Path) -> Result<(), String> {
    fs::create_dir_all(p).map_err(|e| format!("Cannot create save folder: {e}"))?;
    let probe = p.join(format!(".autosnips-check-{}.tmp", std::process::id()));
    let f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|e| format!("Cannot write to this folder: {e}"))?;
    drop(f);
    fs::remove_file(probe).map_err(|e| e.to_string())
}
fn open_path(p: &Path) -> Result<(), String> {
    unsafe {
        if ShellExecuteW(
            std::ptr::null_mut(),
            wide("open").as_ptr(),
            wide(&p.to_string_lossy()).as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        ) as isize
            <= 32
        {
            return Err("Could not open this folder in Explorer".into());
        }
    }
    Ok(())
}
pub fn dialog(s: &str) {
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            wide(s).as_ptr(),
            wide("NativeSnip").as_ptr(),
            MB_OK | MB_ICONERROR,
        );
    }
}
pub fn dispatch(v: &Value) -> Result<Value, String> {
    if v["op"] == "hello" {
        return Ok(json!({"status":"ready","version":env!("CARGO_PKG_VERSION"),"protocol":1}));
    }
    let _ = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    match v["op"].as_str().unwrap_or("") {
        "hello" => Ok(json!({"status":"ready","version":env!("CARGO_PKG_VERSION"),"protocol":1})),
        "ping" => {
            let s = settings()?;
            Ok(
                json!({"status":"ready","version":env!("CARGO_PKG_VERSION"),"protocol":1,"settings":s,"resolved_folder":folder(&s)?.to_string_lossy()}),
            )
        }
        "capture" => capture(settings()?, v),
        "set_settings" => {
            let s: Settings =
                serde_json::from_value(v["settings"].clone()).map_err(|e| e.to_string())?;
            if !matches!(s.launch_method.as_str(), "shortcut" | "protocol") {
                return Err("Choose a valid launch method".into());
            }
            let p = folder(&s)?;
            check_folder(&p)?;
            save_settings(&s)?;
            Ok(
                json!({"status":"ready","settings":s,"resolved_folder":p.to_string_lossy(),"version":env!("CARGO_PKG_VERSION"),"protocol":1}),
            )
        }
        "choose_folder" => unsafe {
            let mut bi: BROWSEINFOW = std::mem::zeroed();
            let title = wide("Choose where NativeSnip saves screenshots");
            bi.lpszTitle = title.as_ptr();
            bi.ulFlags = BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE;
            let pidl = SHBrowseForFolderW(&bi);
            if pidl.is_null() {
                return Ok(json!({"status":"cancelled"}));
            }
            let mut b = [0u16; 32768];
            let ok = SHGetPathFromIDListEx(pidl, b.as_mut_ptr(), b.len() as u32, 0);
            CoTaskMemFree(pidl as *const c_void);
            if ok == 0 {
                return Err("Could not read the selected folder".into());
            }
            let n = b.iter().position(|c| *c == 0).unwrap();
            Ok(json!({"status":"chosen","folder":String::from_utf16_lossy(&b[..n])}))
        },
        "open_folder" => {
            let p = folder(&settings()?)?;
            fs::create_dir_all(&p).map_err(|e| e.to_string())?;
            open_path(&p)?;
            Ok(json!({"status":"opened"}))
        }
        "export_diagnostics" => {
            let r = root()?;
            let export =
                r.join("reports")
                    .join(format!("report-{}-{}", stamp(), std::process::id()));
            fs::create_dir_all(&export).map_err(|e| e.to_string())?;
            for name in ["sessions.jsonl", "sessions.previous.jsonl", "settings.json"] {
                let p = r.join(name);
                if p.exists() {
                    fs::copy(p, export.join(name)).map_err(|e| e.to_string())?;
                }
            }
            fs::write(export.join("README.txt"),"NativeSnip local diagnostics. No screenshot pixels or clipboard text are included. Paths and app format names may appear. Review before sharing. Decision counts do not prove accuracy. Note missed screenshots and unexpected captures separately.\n").map_err(|e|e.to_string())?;
            let timeline = ["sessions.previous.jsonl", "sessions.jsonl"]
                .iter()
                .filter_map(|name| fs::read_to_string(r.join(name)).ok())
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(export.join("timings.csv"), crate::timing::report(&timeline))
                .map_err(|e| e.to_string())?;
            open_path(&export)?;
            Ok(json!({"status":"exported","folder":export.to_string_lossy()}))
        }
        _ => Err("Unknown helper request".into()),
    }
}
fn stamp() -> String {
    unsafe {
        let mut t: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut t);
        format!(
            "{:04}-{:02}-{:02}_{:02}-{:02}-{:02}-{:03}",
            t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, t.wMilliseconds
        )
    }
}
fn log(s: &Settings, v: Value) {
    if !s.diagnostics {
        return;
    }
    if let Ok(r) = root() {
        let p = r.join("sessions.jsonl");
        if fs::metadata(&p)
            .map(|m| m.len() > 5 * 1024 * 1024)
            .unwrap_or(false)
        {
            let _ = fs::remove_file(r.join("sessions.previous.jsonl"));
            let _ = fs::rename(&p, r.join("sessions.previous.jsonl"));
        }
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{v}");
        }
    }
}
unsafe fn owner() -> String {
    window_owner(GetClipboardOwner())
}
unsafe fn window_owner(window: HWND) -> String {
    let mut pid = 0;
    GetWindowThreadProcessId(window, &mut pid);
    if pid == 0 {
        return String::new();
    }
    let p = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if p.is_null() {
        return String::new();
    }
    let mut b = vec![0u16; 32768];
    let mut n = b.len() as u32;
    let ok = QueryFullProcessImageNameW(p, 0, b.as_mut_ptr(), &mut n);
    CloseHandle(p);
    if ok == 0 {
        return String::new();
    }
    String::from_utf16_lossy(&b[..n as usize])
        .rsplit('\\')
        .next()
        .unwrap_or("")
        .to_string()
}
unsafe fn formats() -> Vec<String> {
    let mut f = 0;
    let mut out = vec![];
    loop {
        f = EnumClipboardFormats(f);
        if f == 0 {
            break;
        }
        let mut b = [0u16; 256];
        let n = GetClipboardFormatNameW(f, b.as_mut_ptr(), 256);
        out.push(if n > 0 {
            format!("{f}:{}", String::from_utf16_lossy(&b[..n as usize]))
        } else {
            f.to_string()
        });
        if out.len() >= 128 {
            break;
        }
    }
    out
}
unsafe fn bitmap() -> Result<(Vec<u8>, i32, i32), String> {
    let handle = GetClipboardData(2);
    if handle.is_null() {
        return Err("The captured image is unavailable".into());
    }
    let mut bm: BITMAP = std::mem::zeroed();
    if GetObjectW(
        handle,
        std::mem::size_of::<BITMAP>() as i32,
        &mut bm as *mut _ as *mut c_void,
    ) == 0
    {
        return Err("Cannot read capture dimensions".into());
    }
    let (w, h) = (bm.bmWidth, bm.bmHeight);
    if w <= 0 || h <= 0 {
        return Err("Invalid image dimensions".into());
    }
    let size = (w as usize)
        .checked_mul(h as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| *n <= MAX_BYTES)
        .ok_or("Capture exceeds the 100 MiB decoded image limit")?;
    let mut info: BITMAPINFO = std::mem::zeroed();
    info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    info.bmiHeader.biWidth = w;
    info.bmiHeader.biHeight = -h;
    info.bmiHeader.biPlanes = 1;
    info.bmiHeader.biBitCount = 32;
    info.bmiHeader.biCompression = BI_RGB;
    let mut pixels = vec![0u8; size];
    let dc = CreateCompatibleDC(std::ptr::null_mut());
    if dc.is_null() {
        return Err("Cannot read capture bitmap".into());
    }
    let n = GetDIBits(
        dc,
        handle,
        0,
        h as u32,
        pixels.as_mut_ptr() as *mut c_void,
        &mut info,
        DIB_RGB_COLORS,
    );
    DeleteDC(dc);
    if n != h {
        return Err("Could not read all capture pixels".into());
    }
    Ok((crate::pixels::bgra_to_rgb(&pixels), w, h))
}
fn save(pixels: &[u8], w: i32, h: i32, p: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(p).map_err(|e| e.to_string())?;
    let name = format!("Screenshot_{}_{}.png", stamp(), std::process::id());
    let path = p.join(name);
    let temp = p.join(format!(".autosnips-{}-{}.tmp", stamp(), std::process::id()));
    let mut created = false;
    let result = (|| -> Result<(), String> {
        let f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        created = true;
        let mut buf = BufWriter::new(f);
        {
            let mut enc = png::Encoder::new(&mut buf, w as u32, h as u32);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            let mut wr = enc.write_header().map_err(|e| e.to_string())?;
            wr.write_image_data(pixels).map_err(|e| e.to_string())?;
            wr.finish().map_err(|e| e.to_string())?;
        }
        buf.flush().map_err(|e| e.to_string())?;
        buf.get_ref().sync_all().map_err(|e| e.to_string())?;
        drop(buf);
        unsafe {
            if MoveFileExW(
                wide(&temp.to_string_lossy()).as_ptr(),
                wide(&path.to_string_lossy()).as_ptr(),
                MOVEFILE_WRITE_THROUGH,
            ) == 0
            {
                return Err(
                    "Could not commit screenshot file (destination may already exist)".into(),
                );
            }
        }
        Ok(())
    })();
    if let Err(e) = result {
        if created {
            let _ = fs::remove_file(&temp);
        }
        return Err(e);
    }
    Ok(path)
}
struct Session {
    settings: Settings,
    output: PathBuf,
    last_seq: u32,
    id: String,
    result: Option<Value>,
    retries: u8,
    overlay: isize,
    previous_foreground: isize,
    dismissal: crate::policy::Dismissal,
    started: std::time::Instant,
}
static SESSION: OnceLock<Mutex<Session>> = OnceLock::new();
static WINDOW: std::sync::atomic::AtomicIsize = std::sync::atomic::AtomicIsize::new(0);
unsafe fn finish(hwnd: HWND, v: Value) {
    let mut s = SESSION.get().unwrap().lock().unwrap();
    log(
        &s.settings,
        json!({"version":env!("CARGO_PKG_VERSION"),"session":s.id,"time_local":stamp(),"event":"finished","elapsed_ms":s.started.elapsed().as_millis(),"result":v}),
    );
    s.result = Some(v);
    drop(s);
    DestroyWindow(hwnd);
}
unsafe fn update(hwnd: HWND) {
    let mut s = SESSION.get().unwrap().lock().unwrap();
    let seq = GetClipboardSequenceNumber();
    if seq == s.last_seq {
        return;
    }
    if OpenClipboard(hwnd) == 0 {
        s.retries += 1;
        if s.retries < 10 {
            SetTimer(hwnd, 1, 100, None);
        } else {
            log(
                &s.settings,
                json!({"session":s.id,"time_local":stamp(),"event":"clipboard_busy_timeout","sequence":seq}),
            );
            s.last_seq = seq;
            s.retries = 0;
        }
        return;
    }
    s.retries = 0;
    let source = owner();
    let fmts = formats();
    let image = IsClipboardFormatAvailable(2) != 0
        || IsClipboardFormatAvailable(8) != 0
        || IsClipboardFormatAvailable(17) != 0;
    let kind = crate::policy::classify(&source, &fmts, s.settings.broker_candidates);
    let data = if image && kind.is_some() {
        Some(bitmap())
    } else {
        None
    };
    CloseClipboard();
    s.last_seq = seq;
    log(
        &s.settings,
        json!({"session":s.id,"time_local":stamp(),"event":"clipboard","sequence":seq,"owner":source,"formats":fmts,"decision":if image{kind.unwrap_or("ignored_image_source")}else{"ignored_non_image"}}),
    );
    if let Some(data) = data {
        let response = match data {
            Ok((rgb, w, h)) => match save(&rgb, w, h, &s.output) {
                Ok(path) => {
                    json!({"status":"saved","file":path.file_name().unwrap().to_string_lossy(),"folder":s.output.to_string_lossy(),"width":w,"height":h,"source_rule":kind})
                }
                Err(e) => {
                    json!({"status":"error","message":format!("Screenshot could not be saved: {e}")})
                }
            },
            Err(e) => json!({"status":"error","message":e}),
        };
        drop(s);
        finish(hwnd, response);
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        UPDATE => {
            update(hwnd);
            0
        }
        WM_TIMER if w == 1 => {
            KillTimer(hwnd, 1);
            update(hwnd);
            0
        }
        WM_TIMER if w == 3 => {
            update(hwnd);
            if IsWindow(hwnd) != 0 {
                let mut s = SESSION.get().unwrap().lock().unwrap();
                if s.overlay == 0 {
                    let foreground = GetForegroundWindow();
                    if !foreground.is_null()
                        && foreground as isize != s.previous_foreground
                        && crate::policy::accepted(&window_owner(foreground))
                    {
                        s.overlay = foreground as isize;
                        log(
                            &s.settings,
                            json!({"session":s.id,"event":"capture_window_observed","elapsed_ms":s.started.elapsed().as_millis(),"time_local":stamp()}),
                        );
                    }
                } else if IsWindow(s.overlay as HWND) == 0
                    || IsWindowVisible(s.overlay as HWND) == 0
                {
                    let elapsed = s.started.elapsed().as_millis() as u64;
                    if !s.dismissal.observed() {
                        log(
                            &s.settings,
                            json!({"session":s.id,"event":"capture_window_closed","elapsed_ms":elapsed,"time_local":stamp()}),
                        );
                    }
                    let busy = s.retries != 0;
                    if s.dismissal.tick(false, elapsed, busy) {
                        drop(s);
                        finish(
                            hwnd,
                            json!({"status":"cancelled","message":"Windows capture bar closed. No screenshot was received or saved."}),
                        );
                    }
                } else {
                    let elapsed = s.started.elapsed().as_millis() as u64;
                    s.dismissal.tick(true, elapsed, false);
                }
            }
            0
        }
        WM_TIMER if w == 2 => {
            finish(
                hwnd,
                json!({"status":"timed_out","message":"No new screenshot was received within 90 seconds. Nothing was saved."}),
            );
            0
        }
        CANCEL => {
            finish(
                hwnd,
                json!({"status":"cancelled","message":"Capture session cancelled. Nothing was saved."}),
            );
            0
        }
        WM_DESTROY => {
            RemoveClipboardFormatListener(hwnd);
            WINDOW.store(0, std::sync::atomic::Ordering::SeqCst);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}
unsafe extern "system" fn escape_hook(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && (w == WM_KEYDOWN as usize || w == WM_SYSKEYDOWN as usize) {
        let key = &*(l as *const KBDLLHOOKSTRUCT);
        if key.vkCode == VK_ESCAPE as u32 {
            let hwnd = WINDOW.load(std::sync::atomic::Ordering::SeqCst) as HWND;
            if !hwnd.is_null() {
                PostMessageW(hwnd, CANCEL, 0, 0);
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, w, l)
}
unsafe fn launch_capture(method: &str) -> Result<&'static str, String> {
    let modifiers = [VK_LWIN, VK_RWIN, VK_SHIFT, VK_CONTROL, VK_MENU];
    let held = modifiers
        .iter()
        .any(|key| GetAsyncKeyState(*key as i32) < 0);
    if !held && method != "protocol" {
        let keys = crate::policy::capture_shortcut();
        let input: Vec<INPUT> = keys
            .iter()
            .map(|(key, up)| {
                let mut item: INPUT = std::mem::zeroed();
                item.r#type = INPUT_KEYBOARD;
                item.Anonymous.ki = KEYBDINPUT {
                    wVk: *key,
                    wScan: 0,
                    dwFlags: if *up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                };
                item
            })
            .collect();
        let sent = SendInput(
            input.len() as u32,
            input.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent == input.len() as u32 {
            return Ok("windows_shortcut");
        }
        // Release injected keys after a partial delivery before trying the protocol.
        if sent > 0 {
            let releases: Vec<INPUT> = input.into_iter().skip(3).collect();
            SendInput(
                releases.len() as u32,
                releases.as_ptr(),
                std::mem::size_of::<INPUT>() as i32,
            );
        }
    }
    let n = ShellExecuteW(
        std::ptr::null_mut(),
        wide("open").as_ptr(),
        wide("ms-screenclip:").as_ptr(),
        std::ptr::null(),
        std::ptr::null(),
        SW_SHOWNORMAL,
    ) as isize;
    if n > 32 {
        Ok("screenclip_protocol")
    } else {
        Err("Windows could not open its capture bar. Check that Win+Shift+S works.".into())
    }
}
fn capture(settings: Settings, request: &Value) -> Result<Value, String> {
    let started = std::time::Instant::now();
    unsafe {
        let output = folder(&settings)?;
        // Folder writes are validated when saving, so they do not delay opening Windows UI.
        let mutex = CreateMutexW(
            std::ptr::null(),
            0,
            wide("Local\\AutoSnips.OnDemand.Capture").as_ptr(),
        );
        if mutex.is_null() {
            return Err("Cannot start capture session".into());
        }
        if GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(mutex);
            return Err("Another NativeSnip capture is already running".into());
        }
        let id = format!("{}-{}", stamp(), std::process::id());
        SESSION
            .set(Mutex::new(Session {
                settings,
                output,
                last_seq: GetClipboardSequenceNumber(),
                id,
                result: None,
                retries: 0,
                overlay: 0,
                previous_foreground: GetForegroundWindow() as isize,
                dismissal: crate::policy::Dismissal::default(),
                started,
            }))
            .map_err(|_| "Session already started")?;
        let h = GetModuleHandleW(std::ptr::null());
        let class = wide(CLASS);
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.hInstance = h;
        wc.lpfnWndProc = Some(wndproc);
        wc.lpszClassName = class.as_ptr();
        if RegisterClassW(&wc) == 0 {
            CloseHandle(mutex);
            return Err("Cannot register capture window".into());
        }
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            std::ptr::null_mut(),
            h,
            std::ptr::null(),
        );
        if hwnd.is_null() || AddClipboardFormatListener(hwnd) == 0 {
            CloseHandle(mutex);
            return Err("Cannot listen for screenshot completion".into());
        }
        WINDOW.store(hwnd as isize, std::sync::atomic::Ordering::SeqCst);
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(escape_hook), h, 0); // Escape only; no keyboard contents are logged.
        SetTimer(hwnd, 2, 90000, None);
        SetTimer(hwnd, 3, 150, None);
        let method = SESSION
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .settings
            .launch_method
            .clone();
        let launch = launch_capture(&method);
        let method = match launch {
            Ok(method) => method,
            Err(e) => {
                if !hook.is_null() {
                    UnhookWindowsHookEx(hook);
                }
                DestroyWindow(hwnd);
                CloseHandle(mutex);
                return Err(e);
            }
        };
        {
            let s = SESSION.get().unwrap().lock().unwrap();
            log(
                &s.settings,
                json!({"session":s.id,"event":"launch_requested","method":method,"elapsed_ms":started.elapsed().as_millis(),"browser_ready_ms":request["browser_ready_ms"],"time_local":stamp()}),
            );
        }
        let _ = crate::protocol::write(
            &mut std::io::stdout(),
            &json!({"status":"capturing","message":"Use the Windows capture bar. Press Escape to cancel."}),
        );
        std::thread::spawn(move || loop {
            match crate::protocol::read(&mut std::io::stdin()) {
                Ok(v) if v["op"] == "cancel" => {
                    let hwnd = WINDOW.load(std::sync::atomic::Ordering::SeqCst) as HWND;
                    if !hwnd.is_null() {
                        PostMessageW(hwnd, CANCEL, 0, 0);
                    }
                    break;
                }
                Ok(_) => {}
                Err(_) => {
                    let hwnd = WINDOW.load(std::sync::atomic::Ordering::SeqCst) as HWND;
                    if !hwnd.is_null() {
                        PostMessageW(hwnd, CANCEL, 0, 0);
                    }
                    break;
                }
            }
        });
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let n = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if n <= 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        if !hook.is_null() {
            UnhookWindowsHookEx(hook);
        }
        CloseHandle(mutex);
        let s = SESSION.get().unwrap().lock().unwrap();
        Ok(s.result
            .clone()
            .unwrap_or(json!({"status":"error","message":"Capture ended unexpectedly"})))
    }
}
fn reg(key: &str, value: &str) -> Result<(), String> {
    unsafe {
        let mut h = std::ptr::null_mut();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wide(key).as_ptr(),
            0,
            std::ptr::null(),
            0,
            KEY_SET_VALUE,
            std::ptr::null(),
            &mut h,
            std::ptr::null_mut(),
        ) != 0
        {
            return Err("Could not register browser connection".into());
        }
        let b = wide(value);
        let code = RegSetValueExW(
            h,
            std::ptr::null(),
            0,
            REG_SZ,
            b.as_ptr() as *const u8,
            (b.len() * 2) as u32,
        );
        RegCloseKey(h);
        if code != 0 {
            return Err("Could not save browser connection registration".into());
        }
    }
    Ok(())
}
pub fn register() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().unwrap();
    let chrome = dir.join("chromium-host.json");
    let firefox = dir.join("firefox-host.json");
    fs::write(&chrome,serde_json::to_vec_pretty(&json!({"name":"com.autosnips.capture","description":"AutoSnips native Windows screenshot connection","path":exe.to_string_lossy(),"type":"stdio","allowed_origins":[format!("chrome-extension://{}/",env!("CHROMIUM_EXTENSION_ID"))]})).unwrap()).map_err(|e|e.to_string())?;
    fs::write(&firefox,serde_json::to_vec_pretty(&json!({"name":"com.autosnips.capture","description":"AutoSnips native Windows screenshot connection","path":exe.to_string_lossy(),"type":"stdio","allowed_extensions":[env!("FIREFOX_EXTENSION_ID")]})).unwrap()).map_err(|e|e.to_string())?;
    for prefix in [
        "Software\\Google\\Chrome",
        "Software\\Chromium",
        "Software\\Microsoft\\Edge",
        "Software\\Thorium",
    ] {
        reg(
            &format!("{prefix}\\NativeMessagingHosts\\com.autosnips.capture"),
            &chrome.to_string_lossy(),
        )?;
    }
    reg(
        "Software\\Mozilla\\NativeMessagingHosts\\com.autosnips.capture",
        &firefox.to_string_lossy(),
    )?;
    Ok(())
}

pub fn cancel_running() {
    unsafe {
        let hwnd = FindWindowExW(
            HWND_MESSAGE,
            std::ptr::null_mut(),
            wide(CLASS).as_ptr(),
            std::ptr::null(),
        );
        if !hwnd.is_null() {
            PostMessageW(hwnd, CANCEL, 0, 0);
        }
    }
}

pub fn stop_legacy() {
    unsafe {
        let hwnd = FindWindowExW(
            HWND_MESSAGE,
            std::ptr::null_mut(),
            wide("AutoSnips.MVP.0.1").as_ptr(),
            std::ptr::null(),
        );
        if !hwnd.is_null() {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
        }
    }
}
