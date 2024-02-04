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
