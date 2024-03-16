use std::{fs, path::PathBuf};

use extism_pdk::*;
use proto_pdk::*;

use crate::{
    global_json::GlobalJson,
    helpers::{get_dotnet_root, ANCIENT_VERSIONS},
    release_index::fetch_release_index,
};

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
    fn to_virtual_path(input: String) -> String;
}

static NAME: &str = ".NET";
static BIN: &str = "dotnet";

#[plugin_fn]
pub fn register_tool(Json(_): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadataOutput>> {
    let env = get_host_environment()?;
    let dotnet_sdk_dir = virtual_path!(buf, get_dotnet_root(&env)?.join("sdk"));

    Ok(Json(ToolMetadataOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        plugin_version: Some(env!("CARGO_PKG_VERSION").into()),
        inventory: ToolInventoryMetadata {
            override_dir: Some(dotnet_sdk_dir),
            // we'll stream the output from the dotnet-install script instead
            disable_progress_bars: true,
            ..Default::default()
        },
        ..ToolMetadataOutput::default()
    }))
}

/// Traversing the release index (see fetch_releases_index) would require a lot of requests and since
/// we're in wasm-land where multithreading isn't supported yet that would be very slow.
///
/// So we rely on the published git tags instead which seem to correspond very well to the `sdk.version`
/// field from the release index. A caveat being that this only covers v3+ and has some extra noise to
/// filter out. Versions v1-2 are therefore embedded in the source code.
#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    // we previously used the dotnet/sdk repo here as it included v2, but all prerelease versions were slightly off
    let versions = load_git_tags("https://github.com/dotnet/installer")?
        .iter()
        .filter_map(|tag| tag.strip_prefix("v"))
        .filter(|tag| !tag.is_empty())
        .chain(ANCIENT_VERSIONS.iter().copied())
        .filter_map(|tag| Version::parse(&tag).ok())
        .filter(|version| match &version.pre {
            pre if pre.is_empty() => true,
            pre if pre.starts_with("preview") && !pre.ends_with("sdk") => true,
            pre if pre.starts_with("rc") => true,
            _ => false,
        })
        .collect();

    Ok(Json(LoadVersionsOutput::from_versions(versions)))
}

#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    let mut output = ResolveVersionOutput::default();

    if let UnresolvedVersionSpec::Alias(alias) = &input.initial {
        if alias.eq_ignore_ascii_case("lts") || alias.eq_ignore_ascii_case("sts") {
            let release_index = fetch_release_index()?;
            if let Some(channel) = release_index
                .iter()
                .find(|channel| channel.release_type.eq_ignore_ascii_case(&alias))
            {
                output.version = VersionSpec::parse(&channel.latest_sdk).ok()
            }
        }
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(DetectVersionOutput {
        files: vec!["global.json".into()],
        ignore: vec![],
    }))
}

#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    let mut output = ParseVersionFileOutput::default();

    if input.file == "global.json" {
        if let Ok(global_json) = json::from_str::<GlobalJson>(&input.content) {
            output.version = global_json.sdk.unwrap_or_default().to_version_spec().ok();
        }
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn native_install(
    Json(input): Json<NativeInstallInput>,
) -> FnResult<Json<NativeInstallOutput>> {
    let env = get_host_environment()?;

    let version = &input.context.version;

    let is_windows = env.os.is_windows();
    let script_path = PathBuf::from("/proto/temp").join(if is_windows {
        "dotnet-install.ps1"
    } else {
        "dotnet-install.sh"
    });

    if !script_path.exists() {
        fs::write(
            &script_path,
            fetch(
                HttpRequest::new(if is_windows {
                    "https://dot.net/v1/dotnet-install.ps1"
                } else {
                    "https://dot.net/v1/dotnet-install.sh"
                }),
                None,
            )?
            .body(),
        )?;
    }

    let command_output = exec_command!(
        input,
        ExecCommandInput {
            command: script_path.to_string_lossy().to_string(),
            args: vec![
                "-Version".into(),
                version.to_string(),
                "-InstallDir".into(),
                get_dotnet_root(&env)?
                    .to_str()
                    .ok_or(anyhow!("unable to deduce installation dir"))?
                    .to_owned(),
                "-NoPath".into()
            ],
            set_executable: true,
            stream: true,
            ..ExecCommandInput::default()
        }
    );

    Ok(Json(NativeInstallOutput {
        installed: command_output.exit_code == 0,
        ..NativeInstallOutput::default()
    }))
}

#[plugin_fn]
pub fn native_uninstall(
    Json(_input): Json<NativeUninstallInput>,
) -> FnResult<Json<NativeUninstallOutput>> {
    warn!("This will only uninstall the SDK itself, not the runtime nor any installed workloads.");

    Ok(Json(NativeUninstallOutput {
        uninstalled: true,
        ..NativeUninstallOutput::default()
    }))
}

#[plugin_fn]
pub fn locate_executables(
    Json(_input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;
    let exe_name = env.os.get_exe_name(BIN);
    let mut primary = ExecutableConfig::new(
        &get_dotnet_root(&env)?
            .join(&exe_name)
            .into_os_string()
            .into_string()
            .map_err(|_| anyhow!("unable to build path to the dotnet binary"))?,
    );
    primary.no_bin = true;
    primary.no_shim = true;

    Ok(Json(LocateExecutablesOutput {
        primary: Some(primary),
        ..LocateExecutablesOutput::default()
    }))
}

#[plugin_fn]
pub fn sync_manifest(Json(_): Json<SyncManifestInput>) -> FnResult<Json<SyncManifestOutput>> {
    let env = get_host_environment()?;
    let dotnet_sdk_dir = virtual_path!(buf, get_dotnet_root(&env)?.join("sdk"));

    let mut output = SyncManifestOutput::default();
    let mut versions = vec![];

    // Path may not be whitelisted, so exit early instead of failing
    let Ok(dirs) = fs::read_dir(dotnet_sdk_dir) else {
        warn!("dotnet root is not whitelisted");
        return Ok(Json(output));
    };

    for dir in dirs {
        let dir = dir?.path();

        if !dir.is_dir() {
            continue;
        }

        let name = dir.file_name().unwrap_or_default().to_string_lossy();

        let Ok(spec) = UnresolvedVersionSpec::parse(name) else {
            continue;
        };

        if let UnresolvedVersionSpec::Version(version) = spec {
            versions.push(version);
        }
    }

    if !versions.is_empty() {
        output.versions = Some(versions);
    }

    Ok(Json(output))
}
