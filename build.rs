#[cfg(feature = "html")]
use std::env;
#[cfg(feature = "html")]
use std::fs;
#[cfg(feature = "html")]
use std::path::{Path, PathBuf};

#[cfg(feature = "html")]
fn process_file(path: &Path) {
    // Skip files with the "html" extension.
    if path.extension().map_or(false, |ext| ext == "html") {
        return;
    }
    let content =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
    let mut html = ansi_to_html::convert(&content)
        .unwrap_or_else(|e| panic!("Conversion failed for {:?}: {}", path, e));
    // Add CSS to disable wrapping.
    html = format!("<style>body {{ white-space: pre; }}</style>{}", html);
    // Append ".html" to the original file path.
    let output_path = {
        let mut s = path.to_string_lossy().into_owned();
        s.push_str(".html");
        PathBuf::from(s)
    };
    fs::write(&output_path, html)
        .unwrap_or_else(|e| panic!("Failed to write {:?}: {}", output_path, e));
    println!("Processed: {:?}", output_path);
    // Delete the original file.
    fs::remove_file(path)
        .unwrap_or_else(|e| panic!("Failed to remove original file {:?}: {}", path, e));
}

#[cfg(feature = "html")]
fn process_dir(dir: &Path) {
    for entry in
        fs::read_dir(dir).unwrap_or_else(|e| panic!("Failed to read directory {:?}: {}", dir, e))
    {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            process_dir(&path);
        } else if path.is_file() {
            process_file(&path);
        }
    }
}

#[cfg(feature = "html")]
fn main() {
    // Ensure the build script reruns when files in assets/colorscripts change.
    println!("cargo:rerun-if-changed=assets/colorscripts");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not defined");
    let assets_dir = Path::new(&manifest_dir).join("assets/colorscripts");
    process_dir(&assets_dir);
}

#[cfg(not(feature = "html"))]
fn main() {
    // html feature is not enabled; do nothing.
    println!("cargo:warning=html feature not enabled, skipping html processing.");
}
