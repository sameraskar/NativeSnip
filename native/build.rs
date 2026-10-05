fn main() {
    println!("cargo:rerun-if-changed=../project.json");
    let c: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string("../project.json").expect("project.json"))
            .unwrap();
    println!(
        "cargo:rustc-env=CHROMIUM_EXTENSION_ID={}",
        c["chromium_id"].as_str().unwrap()
    );
    println!(
        "cargo:rustc-env=FIREFOX_EXTENSION_ID={}",
        c["firefox_id"].as_str().unwrap()
    );
}
