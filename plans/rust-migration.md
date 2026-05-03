# Plan: Migrate .claude skills from TypeScript to Rust

## Goal

Surgically adapt all TypeScript-specific skills, agents, and commands to Rust equivalents, preserving the same philosophy (TDD, functional-light, strict types, behaviour-driven testing).

---

## Rust Equivalents Reference

| TypeScript concept | Rust equivalent |
|---|---|
| TypeScript strict mode | `cargo clippy` + `#![deny(clippy::all, clippy::pedantic)]` |
| `tsc --noEmit` | `cargo check` + `cargo clippy` |
| `tsconfig.json` | `Cargo.toml` / `clippy.toml` |
| No `any` | No `unsafe`, no `Box<dyn Any>` without justification |
| Type assertions (`as T`) | `unsafe` transmute / `as` numeric casts with care |
| Zod schemas | `serde` + `garde` / `validator` crates |
| `type` (data shape) | `struct` / `enum` |
| `interface` (behaviour) | `trait` |
| `readonly` | Immutable by default (no `mut`); `Arc<T>` for shared ownership |
| Branded types | Newtype pattern (`struct UserId(String)`) |
| Options objects | Config `struct` with `Default` impl or builder pattern |
| Array methods | Iterator chains (`.map()`, `.filter()`, `.fold()`) |
| `async/await` + Promises | `async/await` + `tokio` / `async-std` |
| Vitest | `cargo test` / `cargo-nextest` |
| Vitest snapshots | `insta` crate for snapshot testing |
| Stryker (mutation testing) | `cargo-mutants` |
| `npm` / `pnpm` | `cargo` |
| React / front-end | N/A — drop |

---

## Files: Heavy Rewrite

These are TypeScript-specific and need full replacement of language, tooling, and code examples.

### 1. `.claude/CLAUDE.md`

- Replace language → Rust (TypeScript strict mode → clippy/deny)
- Replace tools → `cargo test`, `cargo clippy`, `cargo check`
- Replace testing stack → `cargo-nextest` + `insta`
- Replace state management → ownership model, no `mut` by default
- Drop React/Browser Mode references
- Keep: TDD cycle, functional-light, behaviour-driven testing, commit discipline

### 2. `skills/typescript-strict/SKILL.md` → **drop** (no replacement skill)

Rust's compiler and clippy catch most of what this skill existed to enforce. Instead, add a short **"Rust quality gates" section to `CLAUDE.md`** covering:
- Clippy config: `clippy.toml` with `#![deny(clippy::all, clippy::pedantic)]`
- No `.unwrap()` / `.expect()` without a comment justifying why it can't fail
- No `unsafe` blocks without documented invariants
- Avoid gratuitous `.clone()` — it often signals a design problem
- No `#[allow(clippy::...)]` suppression without explanation
- Never silently discard a `Result` (respect `#[must_use]`)

### 3. `agents/ts-enforcer.md` → rename to `agents/rust-enforcer.md`

Full rewrite. New agent:
- Proactively guides toward clippy compliance, newtype pattern, Result/Option
- Reactively scans for `.unwrap()` without justification, `unsafe` without docs, suppressed clippy lints
- Checks for deny attributes or `clippy.toml` in the project
- Validates serde schemas at trust boundaries
- Reports in same severity-tier format (🔴 critical / ⚠️ high / 💡 style)

### 4. `skills/testing/SKILL.md`

Adapt examples from TypeScript/Vitest to Rust:
- Test modules (`#[cfg(test)]`) vs separate test files
- `cargo-nextest` for parallel test execution
- `insta` for snapshot tests (replaces `toMatchInlineSnapshot`)
- Factory functions → Rust builder pattern or helper fns returning structs
- No `beforeEach`/`let` → Rust has no equivalent issue, but same principle: construct in test body
- Keep: test behaviour not implementation, public API only, no 1:1 file mapping

### 5. `skills/mutation-testing/SKILL.md`

Adapt tooling section only (philosophy is unchanged):
- Replace Stryker with `cargo-mutants`
- Commands: `cargo mutants`, `cargo mutants --file src/lib.rs`
- Replace JS/TS code examples with Rust examples
- Keep: RED-GREEN-MUTATE cycle, surviving mutant analysis, report format

### 6. `skills/functional/SKILL.md`

Adapt examples to Rust:
- Immutability → no `mut` by default (stronger than TypeScript's `readonly`)
- Array methods → iterator chains (`.map()`, `.filter()`, `.fold()`, `.collect()`)
- Pure functions → same principle, same rule
- Composition → function chaining, trait combinators
- Drop React-friendly mention
- Keep: no mutation, no comments, options-as-struct, functional-light not category theory

### 7. `skills/characterisation-tests/resources/modern-tooling.md`

Replace Vitest snapshots with `insta`:
- `assert_snapshot!()` → replaces `toMatchInlineSnapshot()`
- `assert_yaml_snapshot!()` for structured data
- `cargo insta review` for reviewing/approving snapshots
- Same workflow: first run captures, subsequent runs compare

### 8. `commands/setup.md`

Adapt detection and generation logic:
- Detect `Cargo.toml` instead of `package.json`
- Detect `nextest`, `clippy.toml`, `deny.toml`
- Generate hooks: run `cargo clippy` after Write/Edit on `.rs` files
- Generate CLAUDE.md with `cargo` commands
- Remove TypeScript config detection section
- Keep: DDD detection, hexagonal detection, 12-factor detection

---

## Files: Drop

Remove these — they are TypeScript/React-specific with no Rust analogue.

- `skills/react-testing/SKILL.md` (and whole `react-testing/` dir)
- `skills/front-end-testing/SKILL.md` (and whole `front-end-testing/` dir)

---

## Files: Light Adaptation

These are mostly language-agnostic. Only update code examples and tool references where TypeScript/JS appears.

- `skills/tdd/SKILL.md` — change `tsc`, `vitest` refs to `cargo check`, `cargo test`
- `skills/hexagonal-architecture/` — swap TS examples for Rust (ports = traits, adapters = impl blocks)
- `skills/domain-driven-design/` — swap TS struct/interface examples for Rust struct/trait/enum
- `skills/api-design/` — swap TS types for Rust structs; keep REST/HTTP principles
- `skills/cli-design/` — swap TS examples for Rust (`clap` crate, `std::process::exit`)
- `skills/finding-seams/` — swap dependency injection examples for Rust trait injection
- `skills/ci-debugging/SKILL.md` — add Cargo-specific CI failure patterns

---

## Files: Keep As-Is

Language-agnostic, no changes needed.

- `skills/diagrams/` — diagramming tools are language-independent
- `skills/planning/SKILL.md`
- `skills/refactoring/SKILL.md` — principles are universal
- `skills/expectations/SKILL.md`
- `skills/find-gaps/SKILL.md`
- `skills/find-skills/SKILL.md`
- `skills/teach-me/`
- `skills/storyboard/SKILL.md`
- `skills/twelve-factor/SKILL.md`
- `skills/test-design-reviewer/SKILL.md` — Dave Farley's properties are language-agnostic
- `agents/adr.md`
- `agents/docs-guardian.md`
- `agents/learn.md`
- `agents/pr-reviewer.md`
- `agents/progress-guardian.md`
- `agents/tdd-guardian.md`
- `agents/twelve-factor-audit.md`
- `agents/use-case-data-patterns.md`
- `agents/refactor-scan.md`
- `commands/continue.md`
- `commands/generate-pr-review.md`
- `commands/plan.md`
- `commands/pr.md`

---

## Execution Order

1. `CLAUDE.md` — establishes new ground truth; includes new "Rust quality gates" section (replaces typescript-strict skill)
2. `skills/typescript-strict/` — delete
3. `agents/ts-enforcer.md` → `agents/rust-enforcer.md` — enforcement agent
4. `skills/testing/SKILL.md` — core testing patterns
5. `skills/mutation-testing/SKILL.md` — tooling swap
6. `skills/functional/SKILL.md` — functional patterns
7. `skills/characterisation-tests/resources/modern-tooling.md` — insta snapshots
8. `commands/setup.md` — onboarding command
9. Light adaptations (tdd, hexagonal, ddd, api-design, cli-design, finding-seams, ci-debugging)
10. Drop react-testing and front-end-testing
