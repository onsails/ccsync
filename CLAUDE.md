# Claude Code Instructions

## Task Master AI Instructions
**Import Task Master's development workflow commands and guidelines, treat as if import is in the main CLAUDE.md file.**
@./.taskmaster/CLAUDE.md

┌─────────────────────────────────────────────────────────────┐
│ ⚠️  MANDATORY BEFORE MARKING ANY TASK COMPLETE:            │
│                                                             │
│ 1. Run: cargo clippy --workspace -- -D warnings            │
│ 2. Run: cargo test --workspace                             │
│ 3. Run: /review-rust-code                                  │
│ 4. Apply all fixes                                         │
│ 5. Only then: task-master set-status --status=done         │
│                                                             │
│ NEVER SKIP THE CODE REVIEW STEP!                           │
└─────────────────────────────────────────────────────────────┘

## Rust Project Standarts

@./.CLAUDE.rust.md

