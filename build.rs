use std::{
    path::PathBuf,
    process::Command,
};

////////////////////////////////////////////////////////////////////////////////

fn main() {
    let mut web_ui_path = PathBuf::from(".");
    web_ui_path.push("web_ui");

    println!("==> Building the web_ui");
    Command::new("cargo")
        .args(&["build"])
        .current_dir(web_ui_path)
        .output()
        .expect("cargo-build failed to execute.");
}
