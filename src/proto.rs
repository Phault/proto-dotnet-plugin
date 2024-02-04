use std::collections::HashMap;

use extism_pdk::*;
use proto_pdk::*;

use crate::release_index::{fetch_channel_releases, fetch_release_index};

static NAME: &str = ".NET";
static BIN: &str = "dotnet";

#[plugin_fn]
pub fn register_tool(Json(_): Json<ToolMetadataInput>) -> FnResult<Json<ToolMetadataOutput>> {
    Ok(Json(ToolMetadataOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
        plugin_version: Some(env!("CARGO_PKG_VERSION").into()),
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
    let output = ParseVersionFileOutput::default();

    if input.file == "global.json" && !input.content.is_empty() {
        // TODO: parse and convert the odd rollForward stuff into a semver range
    }

    Ok(Json(output))
}

#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_host_environment()?;
    check_supported_os_and_arch(
        NAME,
        &env,
        permutations! [
            HostOS::Linux => [
                HostArch::X86, HostArch::X64, HostArch::Arm, HostArch::Arm64
            ],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Windows => [HostArch::X86, HostArch::X64, HostArch::Arm64],
        ],
    )?;

    let version = match input.context.version {
        VersionSpec::Canary => Err(plugin_err!(PluginError::UnsupportedCanary {
            tool: NAME.into(),
        })),
        VersionSpec::Version(v) => Ok(v),
        VersionSpec::Alias(alias) => {
            // uncertain if this code ever runs, as Proto will seemingly always resolve aliases via resolve_version first?

            let releases_index = fetch_release_index()?;

            let channel = match alias.to_lowercase().as_str() {
                "latest" => releases_index.first(),
                "lts" | "sts" => releases_index
                    .iter()
                    .find(|x| x.release_type.eq_ignore_ascii_case(&alias)),
                _ => None,
            };

            match channel {
                Some(c) => Version::parse(&c.latest_sdk).map_err(|e| plugin_err!(e)),
                None => Err(plugin_err!(PluginError::Message(format!(
                    "Alias '{alias}' is not supported"
                )))),
            }
        }
    }?;

    let channel_version = format!("{}.{}", version.major, version.minor);
    let releases = fetch_channel_releases(&channel_version)?;

    let sdk = releases
        .iter()
        .flat_map(|release| {
            release
                .sdks
                .to_owned()
                .unwrap_or(vec![release.sdk.to_owned()])
        })
        .find(|sdk| version.to_string().eq(&sdk.version));

    if sdk.is_none() {
        return Err(plugin_err!(PluginError::Message(
            "Failed to find release matching '{version}'".into()
        )));
    }

    let arch = match env.arch {
        HostArch::X86 => "x86",
        HostArch::X64 => "x64",
        HostArch::Arm => "arm",
        HostArch::Arm64 => "arm64",
        _ => unreachable!(),
    };

    let os = match env.os {
        HostOS::Linux => {
            if is_musl(&env) {
                "linux-musl"
            } else {
                "linux"
            }
        }
        HostOS::MacOS => "osx",
        HostOS::Windows => "win",
        _ => unreachable!(),
    };

    let rid = format!("{os}-{arch}");

    let file_ext = match env.os {
        HostOS::Windows => ".zip",
        _ => ".tar.gz",
    };

    let sdk = sdk.unwrap();
    let file = sdk.files.iter().find(|f| {
        f.rid.as_ref().is_some_and(|file_rid| file_rid.eq(&rid)) && f.name.ends_with(&file_ext)
    });

    match file {
        Some(file) => Ok(Json(DownloadPrebuiltOutput {
            download_url: file.url.to_owned(),
            download_name: file.name.to_owned().into(),
            ..DownloadPrebuiltOutput::default()
        })),
        None => Err(plugin_err!(PluginError::Message(format!(
            "Unable to install {NAME}, unable to find build fitting {rid}."
        )))),
    }
}

#[plugin_fn]
pub fn locate_executables(
    Json(input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    let env = get_host_environment()?;
    let tool_dir = input.context.tool_dir.real_path();

    let exe_name = env.os.get_exe_name(BIN);
    let mut primary = ExecutableConfig::new(&exe_name);
    primary.shim_env_vars = Some(HashMap::from_iter([
        (
            "DOTNET_ROOT".into(),
            tool_dir
                .clone()
                .into_os_string()
                .into_string()
                .unwrap_or_default(),
        ),
        (
            "DOTNET_HOST_PATH".into(),
            tool_dir
                .join(&exe_name)
                .into_os_string()
                .into_string()
                .unwrap_or_default(),
        ),
    ]));

    Ok(Json(LocateExecutablesOutput {
        primary: Some(primary),
        ..LocateExecutablesOutput::default()
    }))
}
