use std::{fs, path::PathBuf};

use extism_pdk::*;
use proto_pdk::*;

use crate::{
    global_json::GlobalJson, helpers::get_dotnet_root, release_index::fetch_release_index,
};

#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

static NAME: &str = ".NET";
static BIN: &str = "dotnet";

#[plugin_fn]
pub fn register_tool(Json(_): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadataOutput>> {
    Ok(Json(ToolMetadataOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        plugin_version: Some(env!("CARGO_PKG_VERSION").into()),
        inventory: ToolInventoryMetadata {
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
/// field from the release index. A caveat being that this only covers v2+ and has some extra noise to
/// filter out.
///
/// Alternatively we could add an option to rely on the slow but reliable method.
#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    let tags = load_git_tags("https://github.com/dotnet/sdk")?
        .iter()
        .filter_map(|tag| tag.strip_prefix("v"))
        .filter(|tag| !tag.is_empty())
        .map(|tag| tag.to_owned())
        .collect::<Vec<_>>();

    let mut versions = vec![];

    for tag in tags {
        let version = Version::parse(&tag);

        match version {
            Ok(v) => {
                // there's a bunch of prereleases which aren't mentioned in the release index, so we filter them out
                if v.pre.is_empty()
                    || (v.pre.starts_with("preview") && !v.pre.ends_with("sdk"))
                    || v.pre.starts_with("rc")
                {
                    versions.push(v);
                }
            }
            _ => {
                debug!("Unable to parse tag '{tag}' as a version");
            }
        }
    }

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
    warn!("Uninstalling .NET sdks is not currently supported, as they all share their installation folder.");

    Ok(Json(NativeUninstallOutput {
        uninstalled: false,
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
