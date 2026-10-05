#![cfg_attr(windows, windows_subsystem = "windows")]
mod pixels;
mod policy;
mod protocol;
mod timing;
#[cfg(windows)]
mod win;
fn authorized(args: &[String]) -> bool {
    let chrome = format!("chrome-extension://{}/", env!("CHROMIUM_EXTENSION_ID"));
    args.iter()
        .skip(1)
        .any(|a| a == &chrome || a == env!("FIREFOX_EXTENSION_ID"))
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    #[cfg(windows)]
    if args.iter().any(|a| a == "--register") {
        if let Err(e) = win::register() {
            win::dialog(&e);
            std::process::exit(1);
        }
        return;
    }
    #[cfg(windows)]
    if args.iter().any(|a| a == "--cancel") {
        win::cancel_running();
        return;
    }
    #[cfg(windows)]
    if args.iter().any(|a| a == "--stop-legacy") {
        win::stop_legacy();
        return;
    }
    if !authorized(&args) {
        return;
    }
    loop {
        let request = match protocol::read(&mut std::io::stdin()) {
            Ok(v) => v,
            Err(_) => return,
        };
        #[cfg(windows)]
        let response = match win::dispatch(&request) {
            Ok(v) => v,
            Err(e) => {
                serde_json::json!({"status":"error","message":e,"version":env!("CARGO_PKG_VERSION"),"protocol":1})
            }
        };
        #[cfg(not(windows))]
        let response = {
            let _ = &request;
            serde_json::json!({"status":"error","message":"Windows only"})
        };
        let ready = request["op"] == "ping" || request["op"] == "hello";
        let _ = protocol::write(&mut std::io::stdout(), &response);
        if !ready {
            break;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_extension_origin() {
        assert!(authorized(&[
            "exe".into(),
            format!("chrome-extension://{}/", env!("CHROMIUM_EXTENSION_ID"))
        ]));
        assert!(authorized(&[
            "exe".into(),
            env!("FIREFOX_EXTENSION_ID").into()
        ]));
        assert!(!authorized(&[
            "exe".into(),
            "chrome-extension://evil/".into()
        ]));
    }
}
