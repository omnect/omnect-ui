use std::path::Path;
use std::process::Command;
use std::{env, io};

fn main() {
    let git_short_rev = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let git_short_rev = git_short_rev.trim();

    println!("cargo:rustc-env=GIT_SHORT_REV={git_short_rev}");

    // Generate embedded static files from Vue dist folder
    generate_static_files().expect("Failed to generate static files");
}

fn generate_static_files() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let generated_file = Path::new(&out_dir).join("generated.rs");

    // Path to the Vue dist folder (relative to backend Cargo.toml)
    let ui_dist_path = Path::new("../ui/dist");

    if ui_dist_path.exists() {
        static_files::resource_dir(ui_dist_path)
            .build()
            .expect("Failed to build static resources");

        println!("cargo:rerun-if-changed=../ui/dist");
    } else {
        // If dist folder doesn't exist, create an empty resource map
        std::fs::write(
            generated_file,
            r#"
use std::collections::HashMap;
pub fn generate() -> HashMap<&'static str, static_files::Resource> {
    HashMap::new()
}
"#,
        )?;
        println!(
            "cargo:warning=UI dist folder not found at {:?}, creating empty resource map",
            ui_dist_path
        );
    }

    Ok(())
}
