use std::{env, path::Path};

fn main() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&cargo_manifest_dir)
            .parent()
            .unwrap()
            .join("lib")
            .display()
    );
}
