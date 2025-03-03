# Spin ghaction Trigger

This is a proof-of-concept [Spin](https://github.com/fermyon/spin) trigger to run GitHub actions using Spin/Wasmtime. This trigger is built on top of [spin-trigger-command](https://github.com/fermyon/spin-trigger-command)

## The end goal

Be able to specify following as an officially supported format by GitHub. (Before you say, I agree that is a stretch. but someone rightly said, first step in achieving something is to dream about it.)

```
name: "get-vault-token"
description: "fetch vault token securely"
runs:
  using: "spin"
  image: "oci://ghcr.io/rajatjindal/get-vault-token:v0.0.46"
inputs:
  workdir:
    description: "Working directory, defaults to env.GITHUB_WORKSPACE"
  vault_url:
    description: "the url to fetch vault token from"

```

## Show me an example

A [test-repo](https://github.com/rajatjindal/wasm-actions/blob/main/.github/workflows/test.yml) demonstrating setting up a tool using this plugin in GitHub actions.

## How it works

WebAssembly is sandboxed by default, and you ask for permissions that you require to run. The idea is that the GitHub actions author will have to ask for specific permissions and this runtime trigger will enable only those capabilities. 

### Outbound requests

By default when you run GitHub action, it could make an outbound request to any URL. With this, he GitHub action author will need to specify which hosts they need to make an outbound request to. 

e.g. If you need to get a token from vault, you need to make an outbound request to your vault url. so you specify in `allowed_outbound_hosts` the URL for that. but the action is not allowed to make any other outbound request therefore a malicious action cannot send your credentials elsewhere.

TODO: document how to specify runtime config for allowed outbound hosts

### Mounts

By default, the GitHub action has access to the complete filesystem. While it may be necessary for some actions, most of them do not need it. With this webassembly based ghaction runner, this is again capability based.

TODO: document how to specify readonly/read-write mounts

### Environment variables

The GitHub action author has to specify which environment variables it needs. The runtime ensures only those environment variables are made available to the guest code.


## Installation

The trigger is installed as a Spin plugin. It can be installed from a release or build.

To install from a release, reference a plugin manifest from a [release](https://github.com/fermyon/spin-trigger-ghaction/releases). For example, to install the canary release:


Goals:

- [ ] Able to run a hello world action
- [ ] Able to download a tool, and add to path
- [ ] Able to download a tool and add it to toolcache and path.
- [ ] Able to download a tool that needs whole dir setup and add it to toolcache and path.
- [ ] Able to get token from a service such as vault


## Installing
```sh
spin plugins install --url https://github.com/rajatjindal/spin-trigger-ghaction/releases/download/canary/trigger-ghaction.json
```

Alternatively, use the `spin pluginify` plugin to install from a fresh build. This will use the pluginify manifest (`spin-pluginify.toml`) to package the plugin and proceed to install it:

```sh
spin plugins install pluginify
cargo build --release
spin pluginify --install
```
