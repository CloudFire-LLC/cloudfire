# Firezone Apple Client

Firezone app clients for macOS and iOS.

## Pre-requisites

- Rust

## Building

1. Clone this repo:

   ```bash
   git clone https://github.com/firezone/firezone
   ```

1. `cd` to the Apple clients code

   ```bash
   cd swift/apple
   ```

1. Rename and populate developer team ID file:

   ```bash
   cp Firezone/xcconfig/Developer.xcconfig.template Firezone/xcconfig/Developer.xcconfig
   vim Firezone/xcconfig/Developer.xcconfig
   ```

1. Open project in Xcode:

```bash
open Firezone.xcodeproj
```

Build the Firezone target

## Debugging

[This Network Extension debugging guide](https://developer.apple.com/forums/thread/725805)
is a great resource to use as a starting point.

### NetworkExtension not loading (macOS)

Try clearing your LaunchAgent db:

```bash
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister -delete
```

**Note**: You MUST reboot after doing this!

### Outdated version of NetworkExtension loading

If you're making changes to the Network Extension and it doesn't seem to be
reflected when you run/debug, it could be that PluginKit is still launching your
old NetworkExtension. Try this to remove it:

```bash
pluginkit -v -m -D -i <bundle-id>
pluginkit -a <path>
pluginkit -r <path>
```

## Cleaning up

Occasionally you might encounter strange issues where it seems like the
artifacts being debugged don't match the code, among other things. In these
cases it's good to clean up using one of the methods below.

### Resetting Xcode package cache

Removes cached packages, built extensions, etc.

```bash
rm -rf ~/Library/Developer/Xcode/DerivedData
```

### Removing build artifacts

To cleanup Swift build objects:

```bash
cd swift/apple
./cleanup.sh
```

To cleanup both Swift and Rust build objects:

```bash
cd swift/apple
./cleanup.sh all
```
