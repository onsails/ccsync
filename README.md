# ccsync

Sync your Claude Code agents, skills, and commands between global and project configurations.

Keep your Claude Code setup consistent across projects while maintaining project-specific customizations.

## 📦 Installation

### Via Cargo

```bash
cargo install ccsync
```

### Via Homebrew

```bash
brew install onsails/homebrew-core/ccsync --build-from-source
```

## 🚀 Quick Start

```bash
# Sync your global Claude settings to current project
ccsync to-local

# Sync project settings back to global
ccsync to-global
```

The tool will prompt you for each file. Press a single key:
- **y** - Yes, sync this file
- **n** - No, skip this file
- **a** - Yes to all remaining
- **s** - Skip all remaining
- **d** - Show me the changes first
- **q** - Quit

### Skip Prompts

```bash
# Auto-approve everything (useful for scripts)
ccsync to-local --yes-all

# Preview what would change (no actual changes)
ccsync to-local --dry-run
```

## 📝 Common Tasks

### Sync Specific Types

```bash
# Sync only agents
ccsync to-local --type=agents

# Sync only skills
ccsync to-local --type=skills

# Sync multiple types
ccsync to-local --type=agents --type=skills
```

### Handling Conflicts

When the same file exists in both locations with different content:

```bash
# Stop and ask (default)
ccsync to-local

# Always overwrite with source
ccsync to-local --conflict=overwrite

# Skip files that have conflicts
ccsync to-local --conflict=skip

# Keep whichever file is newer
ccsync to-local --conflict=newer
```

## 💡 How It Works

By default, `ccsync` asks you to approve each file before syncing:

```
📄 Create new file:
  Source: ~/.claude/agents/test.md
  Dest:   ./.claude/agents/test.md
Proceed? [y/n/a/s/d/q]: _
```

Press **d** to preview the file content before deciding.

Press **a** to approve all remaining files (no more prompts).

Press **q** or **Ctrl+C** to cancel anytime.

## 📂 What Gets Synced

- **Agents** in `~/.claude/agents/` ↔ `./.claude/agents/`
- **Skills** in `~/.claude/skills/` ↔ `./.claude/skills/`
- **Commands** in `~/.claude/commands/` ↔ `./.claude/commands/`

## ⚙️ Configuration Files

Create a `.ccsync` file in your project to customize sync behavior:

```toml
# Ignore certain files (gitignore-style patterns)
ignore = ["**/test-*.md", "**/*.backup"]

# Only sync specific patterns
include = ["agents/**", "skills/**"]

# Set default conflict strategy
conflict_strategy = "newer"
```

**Config file locations** (in order of precedence):
1. `--config <path>` - Custom config file via flag
2. `.ccsync.local` - Project-local (gitignored, for personal settings)
3. `.ccsync` - Project config (committed to repo)
4. `~/.config/ccsync/config.toml` - Global config

**CLI flags always override config files.**

### Skip config files

```bash
# Ignore all config files, use only CLI flags
ccsync to-local --no-config
```

## 💻 Examples

### Check Before Syncing

```bash
# See what would change (no actual sync)
ccsync to-local --dry-run
```

### Sync Only Agents

```bash
# Just sync your agent definitions
ccsync to-local --type=agents --yes-all
```

### Always Use Newer Files

```bash
# Automatically keep whichever file was modified most recently
ccsync to-local --conflict=newer --yes-all
```

## ❓ FAQ

**Q: What happens if I press 'y' on a conflict?**
A: The source file will overwrite the destination (or follow your `--conflict` strategy).

**Q: Can I review all changes before applying them?**
A: Yes! Use `ccsync to-local --dry-run` to preview without making changes.

**Q: What if files are already in sync?**
A: You'll see `Skipped: N (identical content: N)` - no operations performed.

**Q: How do I automate this for scripts?**
A: Use `ccsync to-local --yes-all` to skip all prompts.


## 📄 License

MIT
