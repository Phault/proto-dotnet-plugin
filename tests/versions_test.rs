use proto_pdk_test_utils::*;
use starbase_sandbox::create_empty_sandbox;

generate_resolve_versions_tests!("dotnet-test", {
    "3" => "3.1.426",
    "3.0" => "3.0.103",
    "3.0.103" => "3.0.103",
    "3.0.100-rc1-014190" => "3.0.100-rc1-014190",
    "3.0.100-preview9-014004" => "3.0.100-preview9-014004",
});

#[test]
fn can_load_versions() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("dotnet-test", sandbox.path());

    let output = plugin.load_versions(LoadVersionsInput::default());

    assert!(!output.versions.is_empty());
}

#[test]
fn sets_latest_alias() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("dotnet-test", sandbox.path());

    let output = plugin.load_versions(LoadVersionsInput::default());

    assert!(output.latest.is_some());
    assert!(output.aliases.contains_key("latest"));
    assert_eq!(output.aliases.get("latest"), output.latest.as_ref());
}

// TODO: add tests for all the `rollForward` options
#[test]
fn parses_globaljson() {
    let sandbox = create_empty_sandbox();
    let plugin = create_plugin("dotnet-test", sandbox.path());

    assert_eq!(
        plugin.parse_version_file(ParseVersionFileInput {
            content: r#"{ "sdk": { "version": "8.0.0", "rollForward": "latestFeature" } }"#.into(),
            file: "global.json".into(),
        }),
        ParseVersionFileOutput {
            version: Some(UnresolvedVersionSpec::parse("~8.0.0").unwrap()),
        }
    );
}
