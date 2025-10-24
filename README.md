# ccsync

Bidirectional sync tool for Claude Code configuration files.

## Overview

`ccsync` synchronizes configuration files between your global Claude Code directory (`~/.claude`) and project-local directories (`./.claude`), making it easy to share and manage commands, skills, and subagents across projects.

## Features

- **Bidirectional Sync**: Copy files from global to local (`to-local`) or local to global (`to-global`)
- **Type Filtering**: Sync specific types (commands, skills, subagents) or all at once
- **Conflict Resolution**: Multiple strategies for handling conflicting files
- **Interactive Mode**: Preview and confirm changes before applying
- **Cross-Platform**: Works on Linux, macOS, and Windows

## Installation

### From Source

```bash
git clone https://github.com/yourusername/ccsync
cd ccsync
cargo build --release
```

The binary will be available at `target/release/ccsync`.

## Quick Start

```bash
# Copy global configs to local project
ccsync to-local

# Copy local configs to global
ccsync to-global

# Check sync status
ccsync status

# Show differences
ccsync diff
```

## Project Structure

```
ccsync/
├── src/
│   ├── main.rs          # Entry point and CLI dispatch
│   ├── cli.rs           # CLI argument parsing
│   ├── errors.rs        # Error handling
│   ├── platform.rs      # Cross-platform utilities
│   ├── models/          # Data models
│   │   └── mod.rs
│   └── services/        # Business logic
│       └── mod.rs
├── Cargo.toml
└── README.md
```

## Development

### Requirements

- Rust 1.85 or later (for 2024 edition support)
- Cargo

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with verbose output
cargo run -- --help
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Platform Support

- ✅ Linux
- ✅ macOS  
- ✅ Windows

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
