use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=native=../lib");
    println!("cargo:rustc-link-lib=sai");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I../include")
        .clang_arg("-I../include/sai")
        .clang_arg("-I../include/sai/experimental")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
