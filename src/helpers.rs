use std::path::PathBuf;

use extism_pdk::*;
use proto_pdk::{host_env, HostEnvironment};

#[host_fn]
extern "ExtismHost" {
    fn get_env_var(name: String) -> String;
}

pub fn get_dotnet_root(env: &HostEnvironment) -> Result<PathBuf, Error> {
    // Variable returns a real path
    Ok(host_env!("DOTNET_ROOT")
        .map(PathBuf::from)
        // So we need our fallback to also be a real path
        .unwrap_or_else(|| env.home_dir.real_path().unwrap().join(".dotnet")))
}
