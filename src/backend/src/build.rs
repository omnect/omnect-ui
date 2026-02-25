use std::io;
use std::path::Path;

fn main() {
    // Tell Cargo to only rerun this build script if specific files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../ui/dist");

    // Generate embedded static files from Vue dist folder
    generate_static_files().expect("Failed to generate static files");
}

fn generate_static_files() -> io::Result<()> {
    // Path to the Vue dist folder (relative to backend Cargo.toml)
    let ui_dist_path = Path::new("../ui/dist");

    if !ui_dist_path.exists() {
        panic!(
            "UI dist folder not found at {:?}. Please build the frontend first with: cd ../ui && pnpm run build",
            ui_dist_path
        );
    }

    static_files::resource_dir(ui_dist_path)
        .build()
        .expect("Failed to build static resources");

    Ok(())
}
