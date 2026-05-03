---
name: testing
description: Testing patterns for behavior-driven tests in Rust. Use when writing tests, creating test helpers, structuring test modules, or deciding what to test.
---

# Testing Patterns

For verifying test effectiveness through mutation analysis, load the `mutation-testing` skill. For evaluating test quality against Dave Farley's properties, load the `test-design-reviewer` skill.

## Core Principle

**Test behavior, not implementation.** Full coverage through business behavior, not implementation details.

**Example:** Validation logic in `payment_validator.rs` gets full coverage by testing `process_payment()` behavior, NOT by directly testing validator functions.

---

## Test Through Public API Only

Never test implementation details. Test behavior through public APIs.

**Why this matters:**
- Tests remain valid when refactoring
- Tests document intended behavior
- Tests catch real bugs, not implementation changes

### Examples

❌ **WRONG — Testing implementation:**
```rust
// ❌ Testing a private function directly
#[test]
fn test_validate_cvv() {
    // validate_cvv is pub(crate) — this tests an implementation detail
    assert!(validate_cvv("123"));
}

// ❌ Testing internal state
#[test]
fn test_sets_validated_flag() {
    let mut processor = Processor::new();
    processor.process(payment());
    assert!(processor.is_validated); // internal state
}
```

✅ **CORRECT — Testing behavior through public API:**
```rust
#[test]
fn rejects_negative_amounts() {
    let payment = payment_with(|p| p.amount = -100);
    let result = process_payment(&payment);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PaymentError::InvalidAmount);
}

#[test]
fn rejects_invalid_cvv() {
    let payment = payment_with(|p| p.cvv = "12".into()); // only 2 digits
    let result = process_payment(&payment);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), PaymentError::InvalidCvv);
}

#[test]
fn processes_valid_payment() {
    let payment = default_payment();
    let result = process_payment(&payment);
    assert!(result.is_ok());
    assert!(!result.unwrap().transaction_id.is_empty());
}
```

---

## Coverage Through Behavior

Validation logic gets full coverage by testing the behavior it protects:

```rust
mod process_payment_tests {
    use super::*;

    #[test]
    fn rejects_negative_amounts() {
        let result = process_payment(&payment_with(|p| p.amount = -100));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_amounts_over_limit() {
        let result = process_payment(&payment_with(|p| p.amount = 15_000));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_cvv() {
        let result = process_payment(&payment_with(|p| p.cvv = "12".into()));
        assert!(result.is_err());
    }

    #[test]
    fn processes_valid_payment() {
        let result = process_payment(&default_payment());
        assert!(result.is_ok());
    }
}

// ✅ Result: payment_validator.rs has full coverage through behavior
```

**Key insight:** When coverage drops, ask **"What business behavior am I not testing?"** not "What line am I missing?"

---

## Don't Extract for Testability

Never extract a function into its own module purely to give it its own unit test. Extract for readability, DRY (same **knowledge** in multiple places), or separation of concerns. Not for testability.

If code is inline in a function, it gets coverage through that function's behavioral tests. There is no gap.

The anti-pattern is creating a 1:1 mapping between extracted helpers and test modules. The extracted helper is an implementation detail of its consumer. Test the consumer's behavior.

❌ **WRONG — Extracted single-use helper with its own test:**
```rust
// prepare_participant_data.rs (new file, one caller)
pub fn prepare_participant_data(items: &[Item], user_id: &str) -> ParticipantData {
    ParticipantData {
        your_claims: items.iter().filter(|i| i.is_claimed_by(user_id)).cloned().collect(),
        available: items.iter().filter(|i| !i.is_claimed_by(user_id)).cloned().collect(),
    }
}

// tests — testing the helper directly
#[test]
fn filters_claims() { ... }
```

✅ **CORRECT — Inline in the consuming function, tested through its behavior:**
```rust
pub async fn load_participant_view(db: &Db, event_id: &str, user_id: &str) -> Result<View> {
    let items = get_items(db, event_id).await?;
    let your_claims = items.iter().filter(|i| i.is_claimed_by(user_id)).cloned().collect();
    let available = items.iter().filter(|i| !i.is_claimed_by(user_id)).cloned().collect();
    Ok(View { your_claims, available })
}

#[test]
async fn separates_claimed_and_available() {
    let view = load_participant_view(&db, "event-1", "user-1").await.unwrap();
    assert_eq!(view.your_claims.len(), 1);
    assert_eq!(view.available.len(), 2);
}
```

---

## Test Helper (Factory) Pattern

For test data, use helper functions with optional mutation closures or builder structs.

### Core Principles

1. Return complete, valid objects with sensible defaults
2. Allow per-test customization without shared mutable state
3. Use real types — don't stub or partially construct
4. No shared mutable state across tests (Rust's ownership helps here)

### Basic Pattern — mutation closure

```rust
fn default_user() -> User {
    User {
        id: "user-123".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        role: Role::User,
        is_active: true,
    }
}

fn user_with(f: impl FnOnce(&mut User)) -> User {
    let mut u = default_user();
    f(&mut u);
    u
}

// Usage
#[test]
fn creates_user_with_custom_email() {
    let user = user_with(|u| u.email = "custom@example.com".into());
    assert!(create_user(&user).is_ok());
}
```

### Alternative — builder pattern for complex types

```rust
#[derive(Default)]
struct PaymentBuilder {
    amount: i64,
    currency: String,
    cvv: String,
}

impl PaymentBuilder {
    fn valid() -> Self {
        Self { amount: 100, currency: "GBP".into(), cvv: "123".into() }
    }
    fn amount(mut self, v: i64) -> Self { self.amount = v; self }
    fn cvv(mut self, v: &str) -> Self { self.cvv = v.into(); self }
    fn build(self) -> Payment {
        Payment { amount: self.amount, currency: self.currency, cvv: self.cvv }
    }
}

#[test]
fn rejects_negative_amounts() {
    let payment = PaymentBuilder::valid().amount(-100).build();
    assert!(process_payment(&payment).is_err());
}
```

### Factory Composition

```rust
fn default_order() -> Order {
    Order {
        id: "order-1".into(),
        items: vec![default_item()],
        customer: default_customer(),
        payment: default_payment(),
    }
}

#[test]
fn calculates_total_with_multiple_items() {
    let order = Order {
        items: vec![
            item_with(|i| i.price = 100),
            item_with(|i| i.price = 200),
        ],
        ..default_order()
    };
    assert_eq!(calculate_total(&order), 300);
}
```

### Anti-Patterns

❌ **WRONG: Incomplete objects (missing required fields)**
```rust
fn mock_user() -> User {
    User { id: "user-123".into(), ..Default::default() }
    // Default for name/email may be empty strings — misleading test data
}
```

✅ **CORRECT: All fields explicitly meaningful**
```rust
fn default_user() -> User {
    User {
        id: "user-123".into(),
        name: "Test User".into(),
        email: "test@example.com".into(),
        role: Role::User,
        is_active: true,
    }
}
```

---

## Coverage Theater Detection

### Pattern 1: Asserting a function was called rather than what it produced

❌ **WRONG** — no behavior validation:
```rust
#[test]
fn processes_payment() {
    let mut mock = MockProcessor::new();
    mock.expect_process().times(1).returning(|_| Ok(()));
    handle_payment(&mock, &payment());
    // Only verifies the call happened, not the outcome
}
```

✅ **CORRECT** — verify the outcome:
```rust
#[test]
fn returns_transaction_id_on_success() {
    let result = handle_payment(&real_processor(), &default_payment());
    assert!(result.is_ok());
    assert!(!result.unwrap().transaction_id.is_empty());
}
```

### Pattern 2: Only testing the happy path

❌ **WRONG** — one test, one branch:
```rust
#[test]
fn validates_payment() {
    assert!(validate(&default_payment()).is_ok()); // only happy path
}
```

✅ **CORRECT** — all branches:
```rust
#[test] fn rejects_negative_amounts() { ... }
#[test] fn rejects_amounts_over_limit() { ... }
#[test] fn rejects_invalid_cvv() { ... }
#[test] fn accepts_valid_payment() { ... }
```

### Pattern 3: Testing trivial accessors

❌ **WRONG**:
```rust
#[test]
fn sets_amount() {
    let mut p = Payment::default();
    p.set_amount(100);
    assert_eq!(p.amount(), 100); // trivial round-trip
}
```

✅ **CORRECT** — meaningful behavior:
```rust
#[test]
fn calculates_total_with_tax() {
    let order = order_with_items(&[item_at(100), item_at(100)]);
    assert_eq!(order.total_with_tax(0.15), 230);
}
```

---

## No 1:1 Mapping Between Tests and Implementation

Don't mirror implementation modules with test modules.

❌ **WRONG:**
```
src/
  payment_validator.rs
  payment_processor.rs
  payment_formatter.rs
tests/
  payment_validator_tests.rs  ← 1:1 mapping
  payment_processor_tests.rs  ← 1:1 mapping
  payment_formatter_tests.rs  ← 1:1 mapping
```

✅ **CORRECT:**
```
src/
  payment_validator.rs
  payment_processor.rs
  payment_formatter.rs
tests/
  process_payment.rs  ← tests behavior, not implementation modules
```

In Rust, prefer `#[cfg(test)]` modules inside the source file for unit-level tests, and `tests/` integration tests for public API behavior.

---

## Summary Checklist

When writing tests, verify:

- [ ] Testing behavior through public API (not implementation details)
- [ ] No assertions on whether a function was called — assert on outcomes
- [ ] No tests of private functions or internal state
- [ ] Helper functions return complete, valid objects
- [ ] No shared mutable state across tests
- [ ] Edge cases covered (not just happy path)
- [ ] Tests would pass even if implementation is refactored
- [ ] No 1:1 mapping between test modules and implementation modules
