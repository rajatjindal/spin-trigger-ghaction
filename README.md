# Spin ghaction Trigger

This is a proof-of-concept [Spin](https://github.com/fermyon/spin) trigger to run GitHub actions using Spin/Wasmtime. This trigger is built on top of [spin-trigger-command](https://github.com/fermyon/spin-trigger-command)

## Installation

The trigger is installed as a Spin plugin. It can be installed from a release or build.

To install from a release, reference a plugin manifest from a [release](https://github.com/fermyon/spin-trigger-ghaction/releases). For example, to install the canary release:

```sh
spin plugins install --url https://github.com/rajatjindal/spin-trigger-ghaction/releases/download/canary/trigger-ghaction.json
```

Alternatively, use the `spin pluginify` plugin to install from a fresh build. This will use the pluginify manifest (`spin-pluginify.toml`) to package the plugin and proceed to install it:

```sh
spin plugins install pluginify
cargo build --release
spin pluginify --install
```
