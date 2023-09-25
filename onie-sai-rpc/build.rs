use std::{env, fs, path::PathBuf};

use ttrpc_codegen::{Codegen, Customize, ProtobufCustomize};

fn main() {
    // the OUT_DIR rust build variable path
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // write a `mod.rs` file which makes *both* protobuf and ttrpc code public
    fs::write(
        out_path.join("mod.rs"),
        format!("// @generated\n\npub mod onie_sai;\npub mod onie_sai_ttrpc;\n"),
    )
    .expect("failed to write mod.rs file");

    // our proto files
    let protos = vec!["protos/onie-sai.proto"];

    // we need to disable the generation of this mod.rs file, as we are generating it ourselves above
    let protobuf_customized = ProtobufCustomize::default().gen_mod_rs(false);

    // now generate both protobuf and ttrpc code files
    Codegen::new()
        .out_dir(out_path)
        .inputs(&protos)
        .include("protos")
        .rust_protobuf()
        .customize(Customize {
            ..Default::default()
        })
        .rust_protobuf_customize(protobuf_customized)
        .run()
        .expect("Gen code failed.");
}
