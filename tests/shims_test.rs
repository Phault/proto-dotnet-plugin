use proto_pdk_test_utils::*;

#[cfg(not(windows))]
#[tokio::test]
async fn creates_global_shims() {
    let sandbox = starbase_sandbox::create_empty_sandbox();
    let mut plugin = create_plugin("dotnet-test", sandbox.path());

    plugin.tool.generate_shims(false).await.unwrap();

    assert!(sandbox
        .path()
        .join(".proto/shims")
        .join(if cfg!(windows) {
            "dotnet-test.exe"
        } else {
            "dotnet-test"
        })
        .exists());
}
