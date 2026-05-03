# Development Guidelines for Claude

> **About this file (v4.0.0):** Lean version optimized for context efficiency. Core principles here; detailed patterns loaded on-demand via skills.
>
> **Architecture:**
> - **CLAUDE.md** (this file): Core philosophy + quick reference (~120 lines, always loaded)
> - **Skills**: Detailed patterns loaded on-demand (tdd, testing, mutation-testing, test-design-reviewer, functional, refactoring, expectations, planning, cli-design, finding-seams, characterisation-tests, storyboard, teach-me, diagrams, find-skills, find-gaps, hexagonal-architecture, domain-driven-design, twelve-factor, api-design)
> - **Agents**: Specialized subprocesses for verification and analysis
>
> **Previous versions:**
> - v3.0.0: TypeScript / Vitest stack

## Core Philosophy

**TEST-DRIVEN DEVELOPMENT IS NON-NEGOTIABLE.** Every single line of production code must be written in response to a failing test. No exceptions. This is not a suggestion or a preference - it is the fundamental practice that enables all other principles in this document.

I follow Test-Driven Development (TDD) with a strong emphasis on behavior-driven testing and functional programming principles. All work should be done in small, incremental changes that maintain a working state throughout development.

## Quick Reference

**Key Principles:**

- Write tests first (TDD)
- Test behavior, not implementation
- No `unsafe` without documented invariants
- No `.unwrap()` without justification
- Immutable data by default (no `mut` unless necessary)
- Small, pure functions
- `Result<T, E>` and `Option<T>` over panics

**Preferred Tools:**

- **Language**: Rust (stable)
- **Testing**: `cargo test` / `cargo-nextest` + `insta` for snapshots
- **Mutation testing**: `cargo-mutants`
- **Linting**: `cargo clippy` with strict config
- **Coverage**: `cargo llvm-cov`

## Rust Quality Gates

The Rust compiler catches most type and safety violations automatically. These are the remaining rules that require discipline:

**No `.unwrap()` or `.expect()` without justification**
- If it can't fail by contract, document why: `// SAFETY: vec is non-empty, checked above`
- Prefer `?` operator, pattern matching, or returning `Result`/`Option`

**No `unsafe` without documented invariants**
- Every `unsafe` block must have a `// SAFETY:` comment explaining why it upholds Rust's safety invariants

**No gratuitous `.clone()`**
- Reaching for `.clone()` to satisfy the borrow checker often signals a design problem — rethink ownership first

**No `#[allow(clippy::...)]` without explanation**
- If suppressing a lint, add a comment explaining why the lint doesn't apply here

**Never silently discard a `Result`**
- `let _ = fallible_fn();` is a smell — handle the error or explicitly document why it's safe to ignore

**Clippy config** — projects should have a `clippy.toml` or deny attributes:
```rust
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)] // example of justified suppression
```

## Testing Principles

**Core principle**: Test behavior, not implementation. Full coverage through business behavior.

**Quick reference:**
- Write tests first (TDD non-negotiable)
- Test through public API exclusively
- Use factory functions for test data (no `static mut`, no shared mutable state across tests)
- Tests must document expected business behavior
- No 1:1 mapping between test files and implementation files

For detailed testing patterns and examples, load the `testing` skill.
For verifying test effectiveness through mutation analysis, load the `mutation-testing` skill.

## Code Style

**Core principle**: Functional programming with immutable data. Self-documenting code.

**Quick reference:**
- No data mutation — prefer owned/immutable values, avoid `mut` bindings
- Pure functions wherever possible
- No nested if/else — use early returns (`?`, `return`, `guard` patterns)
- No comments — code should be self-documenting
- Config structs or builder pattern over long positional parameter lists
- Iterator chains (`.map()`, `.filter()`, `.fold()`) over loops

For detailed patterns and examples, load the `functional` skill.

## Development Workflow

**Core principle**: RED-GREEN-MUTATE-KILL MUTANTS-REFACTOR in small, known-good increments. TDD is the fundamental practice.

**Quick reference:**
- RED: Write failing test first (NO production code without failing test)
- GREEN: Write MINIMUM code to pass test
- MUTATE: Run `cargo mutants` to verify test effectiveness, produce a report
- KILL MUTANTS: Address surviving mutants (ask human when value is ambiguous)
- REFACTOR: Assess improvement opportunities (only refactor if adds value)
- **Wait for commit approval** before every commit
- Each increment leaves codebase in working state

For detailed TDD workflow, load the `tdd` skill.
For refactoring methodology, load the `refactoring` skill.
For significant work, load the `planning` skill. Plans live in `plans/` directory.
For CI failure diagnosis, load the `ci-debugging` skill.
For hexagonal architecture projects, load the `hexagonal-architecture` skill.
For Domain-Driven Design projects, load the `domain-driven-design` skill.
For 12-factor service projects, load the `twelve-factor` skill.
For CLI tool design (stream separation, format flags, exit codes, composability), load the `cli-design` skill.
For making untestable code testable, load the `finding-seams` skill.
For documenting existing behavior before changes, load the `characterisation-tests` skill.
For multi-surface design audits before code, load the `storyboard` skill.
For structured learning of any topic (interactive tutoring, courses, quizzes), use `/teach-me [topic]`.
For discovering and installing agent skills from the open ecosystem (`npx skills`), load the `find-skills` skill.
For adversarial review of plans, acceptance criteria, or design mocks, load the `find-gaps` skill.
For relentless plan or design interrogation before implementation, load the `grill-me` skill.

**Project onboarding:** Run `/setup` in any new project to detect its tech stack and generate project-level CLAUDE.md, hooks, commands, and PR review agent in one shot.

**Project-level hooks:** Projects should add a PostToolUse hook in `.claude/settings.json` to run `cargo clippy` after Write/Edit on `.rs` files. Use `/setup` to generate this automatically.

## Output Guardrails

- **Write to files, not chat** — When asked to produce a plan, document, or artifact, always persist it to a file. You may also present it inline for approval, but the file is the source of truth.
- **Plan-only mode** — When asked for a plan, design, or document only, produce ONLY that artifact. Do not write production code, test code, or make any implementation changes unless explicitly asked.
- **Incremental output** — When exploring a codebase, produce a first draft of output within 3-4 tool calls. Refine iteratively rather than front-loading all exploration before producing anything.

## Working with Claude

**Core principle**: Think deeply, follow TDD strictly, capture learnings while context is fresh.

**Quick reference:**
- ALWAYS FOLLOW TDD - no production code without failing test
- Assess refactoring after every green (but only if adds value)
- Update CLAUDE.md when introducing meaningful changes
- Ask "What do I wish I'd known at the start?" after significant changes
- Document gotchas, patterns, decisions, edge cases while context is fresh

For detailed TDD workflow, load the `tdd` skill.
For refactoring methodology, load the `refactoring` skill.
For detailed guidance on expectations and documentation, load the `expectations` skill.

## Resources and References

- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/)
- [cargo-nextest](https://nexte.st/)
- [insta snapshot testing](https://insta.rs/)

## Summary

The key is to write clean, testable, functional code that evolves through small, safe increments. Every change should be driven by a test that describes the desired behavior, and the implementation should be the simplest thing that makes that test pass. When in doubt, favor simplicity and readability over cleverness.
