use std::io::Result;

pub const SOCK_ADDR: &str = r"unix:///run/onie-said.sock";

pub fn remove_sock_addr_if_exist() -> Result<()> {
    let path = SOCK_ADDR.strip_prefix("unix://").unwrap();

    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

include!(concat!(env!("OUT_DIR"), "/mod.rs"));
