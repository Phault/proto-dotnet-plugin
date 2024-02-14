# Changelog

## 0.2.0

#### Install natively using dotnet-install scripts

Having an installation for each .NET SDK version causes all sorts of
issues, as the whole ecosystem expects all SDKs and runtimes to share
the same installation directory.

In practice this is currently causing issues because DOTNET_ROOT cannot
be set to anything meaningful on a system-level. Other tooling like
IntelliJ Rider can thus not find the correct SDK to use. You'd have to
somehow set DOTNET_ROOT per project, and even then that project can only
be aware of a single SDK version at a time.

So I've decided to rely on proto's `native_install` feature instead,
which opens another can of worms, but a more manageable one at least.
It relies on the official dotnet-install scripts to install SDKs into
whatever DOTNET_ROOT is set to (or ~/.dotnet per default).
Supporting uninstallation will come later, as it will be a bit tricky,
now that everything lives in the same directory.

The dotnet executable will no longer be shimmed nor symlinked, and the
user is expected to add DOTNET_ROOT to their PATH.

## 0.1.0

#### global.json detection

The plugin will now try to respect any [global.json](https://learn.microsoft.com/en-us/dotnet/core/tools/global-json) it finds,
but it is done in a best effort manner.

proto will always prefer the highest version you allow it, so the `major`, `minor`, `feature` and `patch` options for the `rollForward` field, will instead act like `latestMajor`, `latestMinor`, `latestFeature` and `latestPatch` respectively.

The `allowPrerelease` field is simply ignored. Due to limitations in proto, or rather it's supporting semver library, this would be troublesome to implement in a faithful way. You may still specify a prerelease in the `version` field however, but how this works together with `rollForward` hasn't been tested.

## 0.0.1

#### 🎉 Release

- Initial release!
