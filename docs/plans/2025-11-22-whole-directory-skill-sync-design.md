# Whole-Directory Skill Syncing Design

## Problem

Currently, `ccsync` only syncs the `SKILL.md` file from skill directories, ignoring supporting files like `assets/`, `references/`, and `scripts/`. Skills like `rust-dev` require these additional files to function properly.

## Solution

Sync entire skill directories instead of just `SKILL.md` files, presenting one user prompt per skill directory.

## Architecture

### 1. Scanner Changes (`scanner/skills.rs`)

**Current:** Returns paths to `SKILL.md` files
**New:** Returns paths to skill directories

```rust
// Before
vec![PathBuf::from("~/.claude/skills/rust-dev/SKILL.md")]

// After
vec![PathBuf::from("~/.claude/skills/rust-dev")]
```

**Logic:**
- Find directories in `skills/` containing `SKILL.md` (validation)
- Return the directory path (not the SKILL.md path)
- Skip directories without `SKILL.md`

### 2. Comparison Layer (`comparison/directory.rs` - new module)

Recursively compare source and destination skill directories:

```rust
struct DirectoryComparison {
    added: Vec<PathBuf>,      // Files in source, not in dest
    modified: Vec<PathBuf>,   // Files changed between source/dest
    removed: Vec<PathBuf>,    // Files in dest, not in source
    unchanged: Vec<PathBuf>,  // Identical files
}
```

**For conflict strategies:**
- `--conflict=newer`: Compare directory modification times (newest file wins)
- `--conflict=overwrite`: Replace entire directory
- `--conflict=skip`: Skip if any file differs

### 3. Diff Display (`comparison/diff.rs`)

File-level diff summary when user presses 'd':

```
📊 Skill directory diff: rust-dev

Files to add:
  + scripts/check.sh
  + assets/logo.png

Files to modify:
  ~ SKILL.md (12 lines changed)
  ~ references/examples.md (3 lines changed)

Files to remove:
  - old_script.sh

Press 'c' to see content diff, or any other key to return...
```

Pressing 'c' shows actual content diffs for modified files.

### 4. Sync Executor (`sync/executor.rs`)

**Type system:**
```rust
enum SyncItem {
    File(PathBuf),
    Directory(PathBuf),
}
```

**Execution:**
- Detect directories vs files
- Copy directories recursively
- Single prompt per skill: "Sync skill `rust-dev`?"
- Apply conflict strategies at directory level

**User prompt format:**
```
📁 Sync skill directory:
  Source: ~/.claude/skills/rust-dev/
  Dest:   ./.claude/skills/rust-dev/
  Files: SKILL.md, assets/, references/, scripts/
Proceed? [y/n/a/s/d/q]: _
```

## Implementation Steps

1. Modify `scanner/skills.rs` to return directories
2. Create `comparison/directory.rs` for recursive comparison
3. Update `comparison/diff.rs` with directory diff display
4. Add `SyncItem` enum to handle files and directories
5. Update `sync/executor.rs` to handle directory syncing
6. Add integration tests for directory syncing
7. Update documentation

## Testing Strategy

- Unit tests: Scanner returns directories with SKILL.md
- Unit tests: Directory comparison logic (added/modified/removed)
- Integration tests: Full skill directory sync workflow
- Edge cases: Empty directories, symlinks, nested structures
- Conflict resolution: Each strategy with directory scenarios
