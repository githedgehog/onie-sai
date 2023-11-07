fn main() -> onie_sai_common::App {
    // match on the zeroth argument and then based on the match of the base path
    // switch to either saictl or said like busybox does it for all their "applets"
    if let Some(arg0) = std::env::args_os().next() {
        // get the basename of arg0
        if let Some(arg0_basename) = std::path::Path::new(&arg0).file_name() {
            // convert arg0_basename to a string
            if let Some(arg0_basename_str) = arg0_basename.to_str() {
                // match on the string
                match arg0_basename_str {
                    "onie-saictl" => return onie_saictl::main(),
                    "onie-said" => return onie_said::main(),
                    _ => return onie_sai_common::App(main_default()),
                }
            }
        }
    }
    onie_sai_common::App(main_default())
}

fn main_default() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let arg0 = std::env::args_os().next();
    let arg0 = arg0
        .as_ref()
        .and_then(|v| std::path::Path::new(v).file_name())
        .and_then(|v| v.to_str())
        .unwrap_or("[none]");
    anyhow::bail!(
        "unsupported app \"{arg0}\", supported apps are \"onie-saictl\" and \"onie-said\""
    )
}
