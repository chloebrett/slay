# Modern Tooling for Characterisation Tests

Rust tooling that supports the characterisation testing workflow. See the main `characterisation-tests` skill for the process and heuristics.

## insta Snapshot Testing

`insta` automates the "let the failure tell you the behavior" step. Instead of manually copying expected values, the framework captures them on first run.

Add to `Cargo.toml`:
```toml
[dev-dependencies]
insta = { version = "1", features = ["ron", "yaml"] }
```

### Inline Snapshots (Preferred for Characterisation)

The expected value lives right in the test file. `insta` fills it in on first run.

```rust
#[test]
fn characterises_format_address() {
    // First run: leave assert_snapshot! with just the value — insta fills it in
    insta::assert_snapshot!(format_address(&test_address()));
}

// After first run, insta rewrites the test (or creates a .snap file):
#[test]
fn characterises_format_address() {
    insta::assert_snapshot!(format_address(&test_address()), @r###"
    123 Main St
    Suite 4B
    Springfield, IL 62701
    "###);
}
```

Inline snapshots are ideal for characterisation because the actual behavior is visible right next to the call.

### File Snapshots

For large outputs (JSON, structured data), `insta` writes a separate `.snap` file:

```rust
#[test]
fn characterises_full_report_output() {
    let report = generate_report(&test_data());
    insta::assert_snapshot!("report_baseline", report);
}
```

This creates `snapshots/characterises_full_report_output__report_baseline.snap`.

### Structured Snapshots

For structs/enums, use YAML or RON snapshots for readable diffs:

```rust
#[test]
fn characterises_order_structure() {
    let order = build_order(&test_cart());
    insta::assert_yaml_snapshot!(order);
}
```

---

## Reviewing Snapshots

```bash
# Review all pending snapshots interactively
cargo insta review

# Accept all pending snapshots
cargo insta accept

# Reject all (discard captured behavior)
cargo insta reject
```

Workflow:
1. Write characterisation test with `assert_snapshot!`
2. Run `cargo test` — test "fails" (no snapshot yet), insta captures the value
3. Run `cargo insta review` to inspect and accept
4. Commit the `.snap` file — it is the documented behavior

---

## cargo-nextest Integration

`cargo-nextest` runs tests faster and gives better output for characterisation test workflows:

```bash
# Run all characterisation tests
cargo nextest run characterise

# Run a specific test
cargo nextest run characterises_format_address
```

---

## When Snapshots Need Updating

After intentional behavior changes, update snapshots:

```bash
# Re-run tests, capturing new values
INSTA_UPDATE=always cargo test
cargo insta review
```

Always review diffs before accepting — the diff is the behavior change.
