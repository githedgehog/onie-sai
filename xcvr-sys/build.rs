use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed=wrapper.h");
    println!(
        "cargo:rerun-if-changed={}",
        Path::new(&cargo_manifest_dir)
            .parent()
            .unwrap()
            .join("include/xcvr/xcvr.h")
            .display()
    );

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I../include")
        .clang_arg("-I../include/xcvr")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_default(true)
        .blocklist_item("XCVR_STATUS_SUCCESS")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
