# PLM - Plugin Manager CLI

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

A unified CLI tool for managing plugins across AI coding assistants (OpenAI Codex, VSCode Copilot). Import Claude Code Plugins and deploy them to other environments. Download, install, and sync Skills, Agents, Prompts, and Instructions seamlessly.

[日本語版 README](README.ja.md)

## Features

- **Multi-Environment Support**: Deploy plugins to OpenAI Codex and VSCode Copilot from a single tool
- **Claude Code Plugin Import**: Import existing Claude Code Plugins and use them in other environments
- **Component Types**: Handle Skills, Agents, Prompts, and Instructions
- **Marketplace Integration**: Browse and install plugins from marketplaces
- **Scope Management**: Install plugins at personal (`~/.codex/`, `~/.copilot/`) or project (`.codex/`, `.github/`) level
- **TUI Interface**: Interactive terminal UI for plugin management
- **Environment Sync**: Keep plugins synchronized across different environments

## Installation

### From Source

```bash
git clone https://github.com/your-org/plugin-manager.git
cd plugin-manager
cargo build --release
```

The binary will be available at `target/release/plm`.

### Requirements

- Rust 2021 edition or later
- Git (for plugin downloads)

## Quick Start

```bash
# Install a plugin from GitHub
plm install owner/repo

# Install with specific target and scope
plm install owner/repo --target codex --scope personal

# List installed plugins
plm list

# Open TUI for plugin management
plm managed
```

## Commands

### Core Commands

| Command | Description | Details |
|---------|-------------|---------|
| `install` | Install plugins | Install from GitHub (`owner/repo`) or marketplace (`plugin@market`) |
| `list` | List installed plugins | Supports `--json`, `--simple`, `--outdated` flags |
| `info` | Show plugin details | View components, author, deployment status |
| `enable` | Enable a plugin | Deploy components from cache to targets |
| `disable` | Disable a plugin | Remove from targets, keep cache |
| `uninstall` | Remove a plugin | Remove completely including cache |
| `update` | Update plugins | Check and apply available updates |

### Configuration Commands

| Command | Description | Details |
|---------|-------------|---------|
| `target` | Manage targets | `list`, `add`, `remove` subcommands |
| `marketplace` | Manage marketplaces | `list`, `add`, `remove`, `update` subcommands |

### Plugin Development

| Command | Description | Details |
|---------|-------------|---------|
| `init` | Generate templates | Create new plugin skeleton |

### Other

| Command | Description | Details |
|---------|-------------|---------|
| `sync` | Sync between environments | Keep plugins synchronized |
| `import` | Import Claude Code Plugins | Convert and deploy existing plugins |
| `managed` | TUI interface | Visual plugin management |

## Usage Examples

### Target Management

```bash
# List configured targets
plm target list

# Add a target environment
plm target add codex

# Remove a target environment
plm target remove copilot
```

### Marketplace Management

```bash
# List registered marketplaces
plm marketplace list

# Add a marketplace
plm marketplace add owner/marketplace-repo

# Update marketplace cache
plm marketplace update
```

### Plugin Installation

```bash
# Install from GitHub repository
plm install owner/repo

# Install specific component types only
plm install owner/repo --type skills --type agents

# Install to specific targets
plm install owner/repo --target codex --target copilot

# Install at project scope
plm install owner/repo --scope project

# Install from marketplace
plm install plugin-name@marketplace-name

# Force re-download (ignore cache)
plm install owner/repo --force
```

### Component Management

```bash
# List all installed components
plm list

# Show component details
plm info component-name

# Enable/disable components
plm enable component-name
plm disable component-name

# Uninstall a component
plm uninstall component-name

# Update components
plm update
```

### Creating Plugins

```bash
# Generate a new plugin template
plm init my-plugin --type skill
```

## Supported Environments

| Environment | Skills | Agents | Prompts | Instructions |
|-------------|:------:|:------:|:-------:|:------------:|
| OpenAI Codex | Yes | - | - | Yes |
| VSCode Copilot | Yes | Yes | Yes | Yes |

## Configuration

PLM stores its configuration at `~/.plm/config.toml`.

### Component Registry

Installed components are tracked in `components.json` for each scope.

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Check (fast compile without binary)
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Third-Party Licenses

A list of third-party license information can be generated using `cargo-about`:

```bash
cargo install --locked cargo-about
cargo about generate --fail -o THIRD_PARTY_LICENSES.md about.md.hbs
```

## License

MIT License - see [LICENSE](LICENSE) file for details.
