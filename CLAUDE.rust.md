## Rust Project Standards

### 1. Dependency Versioning
- Always use requirements: `x.x` (ensures patch compatibility)
- Example: `serde = "1.0"`

### 2. Error Handling - FAIL FAST Principle

**CRITICAL: Logging an error is NOT handling it!**

- **NEVER** just log errors and continue execution
- **NEVER** return `Ok(())` after encountering an error
- **ALWAYS** propagate errors up the stack with `?` or explicit return
- The program MUST halt when errors occur, not stumble forward

**❌ FORBIDDEN - These are all error swallowing:**
```rust
// WRONG: Logs but continues
if let Err(e) = operation() {
    log::error!("Failed: {}", e);  // Still swallowing!
}
// Continues execution...

// WRONG: Prints but returns success
match operation() {
    Ok(val) => process(val),
    Err(e) => {
        eprintln!("Error: {}", e);  // Still swallowing!
        return Ok(());  // NEVER do this!
    }
}

// WRONG: Counts errors but continues
Err(e) => {
    error_count += 1;  // Still swallowing!
    log::warn!("Error #{}: {}", error_count, e);
}

// WRONG: unwrap_or* silently swallows errors
let val = operation().unwrap_or_default();  // Still swallowing!
let val = operation().unwrap_or(fallback);  // Still swallowing!
let val = operation().unwrap_or_else(|_| fallback);  // Still swallowing!
```

**✅ REQUIRED - Propagate ALL errors:**
```rust
operation()?;  // Propagates error, halts execution

// Or explicitly:
match operation() {
    Ok(val) => process(val),
    Err(e) => return Err(e.into()),  // Propagate, don't swallow!
}
```

### 3. Error Types
- **Library crates/modules**: Use `thiserror` with backtrace support
- **Binary main.rs & tests**: Use `anyhow`
- **Other derives**: Use `derive_more` (Display, From, Into, etc.)

### 4. Workspace Architecture
- Always use Cargo workspace with single-responsibility crates
- Root `Cargo.toml` defines workspace, contains no code
- CLI must be separate subcrate
- Structure: `project/`, `project-cli/`, `project-client/`, etc.

### 5. Testing

#### 5.1 Unit Tests
- **NEVER** use `std::env::set_var()` in tests (pollutes environment)
- **ALWAYS** pass config through function parameters
- Tests in same file using `#[cfg(test)]` module

#### 5.2 Integration Tests - MANDATORY for Coordinated Systems

**CRITICAL: Unit tests alone give false confidence for multi-component systems**

**The Gap:**
- Unit tests verify components "CAN" work (capabilities tested in isolation)
- Integration tests verify components "DOES" coordinate (actual runtime behavior)
- Passing all unit tests ≠ working system

**Integration tests are REQUIRED when:**
- Components communicate via channels (mpsc, watch, broadcast)
- Shared state between components (Arc<RwLock<>>, Arc<Mutex<>>)
- Event-driven coordination (async tasks, spawned workers)
- Trait implementations that interact across boundaries
- Factory patterns that wire multiple components together

**Critical Design Rule:**
- **NEVER manually trigger coordination mechanisms in integration tests**
- Use timeouts to catch missing notifications/updates (fail fast on missing coordination)
- Test the full end-to-end path, not just individual capabilities
- If two components share a channel/lock/watch, you MUST have an integration test proving they actually coordinate

**What to verify:**
- Component A action → Component B receives expected effect
- Shared state updates propagate correctly
- Event ordering guarantees hold
- Shutdown cleans up all resources
- No race conditions under concurrent access

### 6. Configuration Management
- **CLI-First**: Never bypass CLI argument parsing
- **NEVER** use `Default` trait that reads environment
- **ALWAYS** use `from_cli_args()` factory methods
- Config flows: CLI args → Config struct → Client

### 7. Python Helper Scripts
- Location: `helpers/` directory
- Initialize: `uv init helpers/`
- **ALWAYS** use `uv add <package>` (NEVER `uv pip install`)

### 8. Code Standards
- **Visibility**: Private (default) > pub(crate) > pub
- **Magic Numbers**: Use `const` or CLI args, never literals
- **Async**: Use tokio consistently
- **Breaking Changes**: OK for internal crates, preserve HTTP/WebSocket compatibility

### 9. Rust Versioning
- **Cargo edition**: Use 2024 edition

