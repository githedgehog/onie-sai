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

pub fn wrap_message_field<T>(obj: Option<T>) -> protobuf::MessageField<T> {
    match obj {
        Some(obj) => protobuf::MessageField::some(obj),
        None => protobuf::MessageField::none(),
    }
}
