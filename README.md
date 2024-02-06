# proto .NET plugin

[.NET](https://dotnet.microsoft.com/) WASM plugin for [proto](https://github.com/moonrepo/proto).

## Installation

This is a community plugin and is thus not built-in to proto. In order to use it, add the following to `.prototools`:

```toml
[plugins]
dotnet = "source:https://github.com/Phault/proto-dotnet-plugin/releases/latest/download/dotnet_plugin.wasm"
```

Or preferably pin a specific version, to avoid nasty surprises if we mess up a release:

```toml
[plugins]
dotnet = "source:https://github.com/Phault/proto-dotnet-plugin/releases/download/vX.Y.Z/dotnet_plugin.wasm"
```

## Usage

```shell
# install latest SDK
proto install dotnet

# install latest long-term-support release
proto install dotnet lts

# install a specific version
proto install dotnet 8.0.101
```

## Caveats

### global.json detection

The plugin will try to respect any [global.json](https://learn.microsoft.com/en-us/dotnet/core/tools/global-json) it finds,
but it is done in a best effort manner.

proto will always prefer the highest version you allow it, so the `major`, `minor`, `feature` and `patch` options for the `rollForward` field, will instead act like `latestMajor`, `latestMinor`, `latestFeature` and `latestPatch` respectively.

The `allowPrerelease` field is simply ignored. Due to limitations in proto, or rather it's supporting semver library, this would be troublesome to implement in a faithful way. You may still specify a prerelease in the `version` field however, but how this works together with `rollForward` hasn't been tested.

## Configuration

.NET plugin does not support configuration.

## Hooks

.NET plugin does not support hooks.

## Contributing

Build the plugin:

```shell
cargo build --target wasm32-wasi
```

Test the plugin by running `proto` commands.

```shell
proto install dotnet-test
proto list-remote dotnet-test
```
