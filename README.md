# ccsync - Claude Configuration Synchronization Tool

A powerful CLI tool for synchronizing agents, skills, and commands between global (`~/.claude`) and project-specific (`./.claude`) directories.

## Features

- 🔄 **Bidirectional Sync** - Sync from global to local or local to global
- 💬 **Interactive Mode** - Single-key shortcuts for approving each file (default)
- ⚡ **Automation** - Non-interactive mode with `--yes-all` flag
- 🔍 **Preview** - Dry-run mode to see changes before applying
- 🎯 **Selective Sync** - Filter by type (agents, skills, commands)
- ⚠️ **Conflict Resolution** - Multiple strategies (fail, overwrite, skip, newer)
- 📊 **Smart Reporting** - Clear summaries with skip reasons
- 🔗 **Symlink Support** - Handles NixOS and other symlinked configs
- ⌨️ **Fast Workflow** - Single keypress actions, no Enter required

## Installation

```bash
cargo install --path crates/ccsync-cli
```

Or build from source:

```bash
cargo build --release
# Binary available at: target/release/ccsync
```

## Quick Start

### Interactive Sync (Default)

```bash
# Sync global → local with prompts
ccsync to-local

# Example interaction:
📄 Create new file:
  Source: /home/user/.claude/agents/test.md
  Dest:   /home/user/project/.claude/agents/test.md
Proceed? [y/n/a/s/d/q] (yes/no/all/skip-all/diff/quit): y
```

### Keyboard Shortcuts

Press a single key for instant action (no Enter required):

- **y** - Approve this file
- **n** - Skip this file (default)
- **a** - Approve **all** remaining files
- **s** - Skip **all** remaining files
- **d** - Show diff or file content
- **q** - Quit cleanly

### Non-Interactive Modes

```bash
# Auto-approve all (for automation/scripts)
ccsync to-local --yes-all

# Preview changes without applying
ccsync to-local --dry-run

# Combine flags
ccsync to-local --type=agents --conflict=skip --dry-run
```

## Usage

### Basic Commands

```bash
# Sync global → local (interactive)
ccsync to-local

# Sync local → global (interactive)
ccsync to-global

# Show sync status
ccsync status

# Show differences
ccsync diff

# Show active configuration
ccsync config
```

### Filtering by Type

```bash
# Sync only agents
ccsync to-local --type=agents

# Sync multiple types
ccsync to-local --type=agents --type=skills

# Sync everything
ccsync to-local --type=all
```

### Conflict Resolution

When files exist in both locations with different content:

```bash
# Abort on conflicts (default)
ccsync to-local --conflict=fail

# Overwrite destination with source
ccsync to-local --conflict=overwrite

# Skip conflicting files
ccsync to-local --conflict=skip

# Keep newer file based on timestamp
ccsync to-local --conflict=newer
```

### Global Flags

```bash
# Verbose output
ccsync --verbose to-local

# Auto-approve all (non-interactive)
ccsync --yes-all to-local

# Preview changes (dry-run)
ccsync --dry-run to-local

# Combine flags
ccsync --verbose --dry-run to-local --type=agents
```

## Interactive Mode

### Default Behavior

By default, `ccsync` runs in **interactive mode**, prompting you to approve each sync action:

```
📄 Create new file:
  Source: /home/user/.claude/agents/test.md
  Dest:   /home/user/project/.claude/agents/test.md
Proceed? [y/n/a/s/d/q] (yes/no/all/skip-all/diff/quit):
```

### Session State

- Press **a** (all): Remaining files are auto-approved
- Press **s** (skip-all): Remaining files are auto-skipped
- Choice persists for the current session only

### Viewing Diffs

Press **d** to see what will change:

**For new files:**
```
--- New file ---
+++ /path/to/file.md
+# Agent Title
+Agent content here
+All shown as additions
```

**For conflicts:**
```
--- /path/to/dest.md
+++ /path/to/source.md
@@ -1,3 +1,3 @@
 Line 1
-Old content
+New content
 Line 3
```

### Exiting

- Press **q**: Clean exit with message "Sync cancelled by user."
- Press **Ctrl+C**: Graceful interrupt (exit code 130)

## Configuration

### Directory Structure

```
~/.claude/                    # Global configuration
├── agents/                   # Agent definitions (*.md)
├── skills/                   # Skills (*/SKILL.md)
└── commands/                 # Commands (*.md)

project/.claude/              # Project-specific configuration
├── agents/
├── skills/
└── commands/
```

### Config Files (Future)

Configuration file support will be added in a future release. Currently, all options are specified via CLI flags.

## Output Examples

### Successful Sync

```
=== Sync Summary ===
Created:  5
Updated:  2
Deleted:  0
Skipped:  3 (identical content: 3)
Conflicts: 0

Total operations: 7
Status: ✓ Success
```

### Dry-Run Preview

```bash
$ ccsync to-local --dry-run

[DRY RUN] Would create: /project/.claude/agents/test.md
[DRY RUN] Would update: /project/.claude/skills/skill1/SKILL.md

=== Sync Summary ===
Created:  1
Updated:  1
Deleted:  0
Skipped:  0
Conflicts: 0

Total operations: 2
Status: ✓ Success
```

### Skip Reasons

The tool provides clear explanations for skipped files:

```
Skipped:  5 (identical content: 4, user skipped: 1)
```

## Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test --workspace

# Run clippy
cargo clippy --workspace -- -D warnings
```

### Project Structure

```
ccsync/
├── crates/
│   ├── ccsync/              # Core library
│   │   ├── comparison/      # File comparison and diffs
│   │   ├── config/          # Configuration management
│   │   ├── scanner/         # File scanning
│   │   └── sync/            # Sync engine
│   └── ccsync-cli/          # CLI binary
│       ├── commands/        # Command handlers
│       └── interactive/     # Interactive prompts
├── tests/                   # Integration tests
└── Cargo.toml              # Workspace configuration
```

### Architecture

- **Library crate** (`ccsync`): Core sync logic, platform-agnostic
- **CLI crate** (`ccsync-cli`): User interface and command handling
- **Separation of concerns**: UI logic separate from business logic

## Exit Codes

- **0**: Success or user-initiated cancellation
- **1**: Error (I/O failures, conflicts with fail strategy)
- **130**: Interrupted by Ctrl+C (SIGINT)

## Platform Support

- ✅ Linux
- ✅ macOS
- ✅ Windows
- ✅ NixOS (handles symlinked configurations)

Detects home directory using `$HOME` (Unix) or `%USERPROFILE%` (Windows).

## Examples

### Sync Workflow

```bash
# 1. Check what would change
ccsync to-local --dry-run

# 2. Sync with interactive prompts
ccsync to-local

# Press 'd' to view a file before approving
# Press 'y' to approve
# Press 'a' to approve all remaining

# 3. View results
=== Sync Summary ===
Created:  8
Updated:  0
Skipped:  0 (user skipped: 0)
```

### Automation Scripts

```bash
#!/bin/bash
# Non-interactive sync for CI/CD

ccsync to-local --yes-all --type=agents --conflict=overwrite
if [ $? -eq 0 ]; then
    echo "Sync successful"
else
    echo "Sync failed"
    exit 1
fi
```

### Selective Sync

```bash
# Sync only agents with overwrite strategy
ccsync to-local --type=agents --conflict=overwrite

# Sync agents and skills, skip on conflicts
ccsync to-global --type=agents --type=skills --conflict=skip
```

## Troubleshooting

### "Failed to strip prefix" Error

If you encounter symlink path issues (common on NixOS):
- The tool automatically handles symlinked configs
- If issues persist, check that source directories exist

### No Changes Detected

If sync reports 0 operations:
```
Skipped:  8 (identical content: 8)
```

This means files are already synchronized (same content in both locations).

### Interactive Mode Not Working

If running in a non-TTY environment (CI/CD, scripts):
- Use `--yes-all` flag for automation
- Or use `--dry-run` for preview only

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test --workspace`
2. Clippy is clean: `cargo clippy --workspace -- -D warnings`
3. Code is formatted: `cargo fmt --all`

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [dialoguer](https://github.com/console-rs/dialoguer) - Interactive prompts
- [similar](https://github.com/mitsuhiko/similar) - Diff generation
- [ignore](https://github.com/BurntSushi/ripgrep/tree/master/crates/ignore) - Pattern matching

---

**Status**: Active development 🚀
**Version**: 0.1.0
