# Changelog

## 0.1.0

#### global.json detection

The plugin will now try to respect any [global.json](https://learn.microsoft.com/en-us/dotnet/core/tools/global-json) it finds,
but it is done in a best effort manner.

proto will always prefer the highest version you allow it, so the `major`, `minor`, `feature` and `patch` options for the `rollForward` field, will instead act like `latestMajor`, `latestMinor`, `latestFeature` and `latestPatch` respectively.

The `allowPrerelease` field is simply ignored. Due to limitations in proto, or rather it's supporting semver library, this would be troublesome to implement in a faithful way. You may still specify a prerelease in the `version` field however, but how this works together with `rollForward` hasn't been tested.

## 0.0.1

#### 🎉 Release

- Initial release!
