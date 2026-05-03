---
name: functional
description: Functional programming patterns with immutable data in Rust. Use when writing logic, data transformations, or encountering mutation bugs. Covers immutability, pure functions, composition, early returns, and config structs. Do NOT over-apply heavy FP abstractions unless the project requires them.
---

# Functional Patterns

## Core Principles

- **No data mutation** — immutable by default, `mut` only when necessary
- **Pure functions** wherever possible
- **Composition** over inheritance
- **No comments** — code should be self-documenting
- **Iterator chains** over loops
- **Config structs** over long positional parameter lists

---

## Why Immutability Matters

Rust makes immutability the default — bindings are immutable unless declared `mut`. Lean into this.

- **Predictable**: Same input always produces same output (no hidden state changes)
- **Debuggable**: State doesn't change unexpectedly — easier to trace bugs
- **Testable**: No hidden mutable state makes tests straightforward
- **Concurrency-safe**: `Send + Sync` bounds are easier to satisfy without interior mutability

**Example of the problem:**
```rust
// ❌ WRONG — mutation creates unpredictable behavior
fn grant_permission(user: &mut User, permission: &str) {
    user.permissions.push(permission.to_string()); // mutates caller's data
}

let mut user = User { permissions: vec!["read".into()] };
grant_permission(&mut user, "write");
// user has silently changed — caller may not expect this
```

```rust
// ✅ CORRECT — return a new value
fn grant_permission(user: &User, permission: &str) -> User {
    let mut permissions = user.permissions.clone();
    permissions.push(permission.to_string());
    User { permissions, ..user.clone() }
}

let user = User { permissions: vec!["read".into()] };
let updated = grant_permission(&user, "write");
// user is unchanged; updated is the new version
```

---

## Functional Light

We follow "Functional Light" principles — practical functional patterns without heavy abstractions:

**What we DO:**
- Pure functions and immutable data
- Composition and declarative code
- Iterator chains over loops
- `Result<T, E>` and `Option<T>` for error/absence handling

**What we DON'T do:**
- Category theory or monad stacks
- Heavy FP crates (unless the project explicitly calls for them)
- Over-engineering with abstractions
- Functional for the sake of functional

**Why:** The goal is **maintainable, testable code** — not academic purity. If a functional pattern makes code harder to understand, don't use it.

**Example — keep it simple:**
```rust
// ✅ GOOD — simple, clear, functional
let active_users: Vec<_> = users.iter().filter(|u| u.active).collect();
let user_names: Vec<_> = active_users.iter().map(|u| &u.name).collect();

// ❌ OVER-ENGINEERED — unnecessary abstraction
fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |x| g(f(x))
}
```

---

## No Comments / Self-Documenting Code

Code should be clear through naming and structure.

**Exception**: `// SAFETY:` comments on `unsafe` blocks (required), and doc comments (`///`) for public APIs.

### Examples

❌ **WRONG — comments explaining unclear code**
```rust
// check if user can do the thing
fn chk(u: &Option<User>) -> bool {
    // see if user exists
    if let Some(u) = u {
        // check if active
        if u.a {
            // check permission
            if u.p { return true; }
        }
    }
    false
}
```

✅ **CORRECT — self-documenting code**
```rust
fn can_user_access_resource(user: Option<&User>) -> bool {
    user.map_or(false, |u| u.is_active && u.has_permission)
}
```

✅ **Acceptable doc comment for public API**
```rust
/// Registers a scenario for runtime switching.
///
/// # Errors
/// Returns [`Error::DuplicateId`] if a scenario with this ID already exists.
pub fn register_scenario(definition: ScenarioDefinition) -> Result<(), Error> {
    // ...
}
```

---

## Iterator Chains Over Loops

Prefer `.map()`, `.filter()`, `.fold()` for transformations. They're declarative and work naturally with Rust's ownership model.

### Map — transform each element

❌ **WRONG — imperative loop**
```rust
let mut scenario_ids = Vec::new();
for scenario in &scenarios {
    scenario_ids.push(scenario.id.clone());
}
```

✅ **CORRECT — iterator map**
```rust
let scenario_ids: Vec<_> = scenarios.iter().map(|s| s.id.clone()).collect();
```

### Filter — select subset

❌ **WRONG**
```rust
let mut active = Vec::new();
for scenario in &scenarios {
    if scenario.active {
        active.push(scenario);
    }
}
```

✅ **CORRECT**
```rust
let active: Vec<_> = scenarios.iter().filter(|s| s.active).collect();
```

### Fold — aggregate values

❌ **WRONG**
```rust
let mut total = 0.0;
for item in &items {
    total += item.price * item.quantity as f64;
}
```

✅ **CORRECT**
```rust
let total: f64 = items.iter().fold(0.0, |sum, item| sum + item.price * item.quantity as f64);
// or with sum():
let total: f64 = items.iter().map(|item| item.price * item.quantity as f64).sum();
```

### Chaining multiple operations

✅ **CORRECT — compose iterators**
```rust
let total: f64 = items
    .iter()
    .filter(|item| item.active)
    .map(|item| item.price * item.quantity as f64)
    .sum();
```

### When loops are acceptable

- Early termination with complex state (use `for` + `break` or `.find()` / `.any()`)
- Side effects that consume ownership
- Async iteration (iterator adapters don't support `async` directly)

---

## Config Structs Over Long Parameter Lists

Default to a config struct when a function takes 3+ parameters. This improves readability and allows optional fields via `Default`.

### Why config structs?

- Named fields — clear what each argument means
- No ordering dependencies
- Optional fields with `Default::default()`
- Easy to extend without breaking callers

### Examples

❌ **WRONG — positional parameters**
```rust
fn create_payment(
    amount: u64,
    currency: &str,
    card_id: &str,
    cvv: &str,
    save_card: bool,
    send_receipt: bool,
) -> Payment { ... }

// Call site — unclear what each argument means
create_payment(100, "GBP", "card_123", "123", true, false);
```

✅ **CORRECT — config struct**
```rust
#[derive(Default)]
struct CreatePaymentOptions<'a> {
    amount: u64,
    currency: &'a str,
    card_id: &'a str,
    cvv: &'a str,
    save_card: bool,
    send_receipt: bool,
}

fn create_payment(opts: CreatePaymentOptions<'_>) -> Payment { ... }

// Call site — crystal clear
create_payment(CreatePaymentOptions {
    amount: 100,
    currency: "GBP",
    card_id: "card_123",
    cvv: "123",
    save_card: true,
    ..Default::default()
});
```

### When positional parameters are OK

- 1–2 parameters
- Order is obvious (`add(a, b)`)
- High-frequency utility functions

---

## Pure Functions

Pure functions have no side effects and always return the same output for the same input.

### What makes a function pure?

1. **No side effects** — no mutations of external state, no I/O
2. **Deterministic** — same input → same output, no dependency on global state
3. **Referentially transparent** — you can substitute the call with its return value

### Examples

❌ **WRONG — impure (side effect)**
```rust
static mut COUNT: u64 = 0;

fn increment() -> u64 {
    unsafe { COUNT += 1; COUNT } // ❌ mutates global state
}
```

✅ **CORRECT — pure**
```rust
fn increment(count: u64) -> u64 {
    count + 1 // ✅ no external state
}
```

### Isolating impure functions

Keep impure functions (I/O, randomness, time) at system boundaries:

```rust
// ✅ Pure core
fn calculate_total(items: &[Item]) -> u64 {
    items.iter().map(|i| i.price).sum()
}

// ✅ Impure shell — isolated at the edge
async fn save_order(order: &Order, db: &Database) -> Result<(), Error> {
    let total = calculate_total(&order.items); // pure
    db.save(order.id, total).await             // impure (I/O)
}
```

---

## Result Type for Error Handling

Use `Result<T, E>` and `Option<T>` — never panic in library code.

```rust
// ✅ Return Result, let callers decide how to handle
fn process_payment(payment: &Payment) -> Result<Transaction, PaymentError> {
    if payment.amount == 0 {
        return Err(PaymentError::InvalidAmount);
    }
    Ok(execute_payment(payment))
}

// ✅ Use ? to propagate
fn handle_checkout(cart: &Cart, payment: &Payment) -> Result<Order, CheckoutError> {
    let transaction = process_payment(payment)?;
    let order = create_order(cart, transaction)?;
    Ok(order)
}
```

---

## Early Returns Over Nesting

```rust
// ❌ WRONG — deeply nested
fn process_order(order: &Order) -> Result<(), Error> {
    if !order.items.is_empty() {
        if order.customer.verified {
            if order.total > 0 {
                // ... logic buried here
            }
        }
    }
    Ok(())
}

// ✅ CORRECT — guard clauses with early returns
fn process_order(order: &Order) -> Result<(), Error> {
    if order.items.is_empty() { return Err(Error::EmptyOrder); }
    if !order.customer.verified { return Err(Error::UnverifiedCustomer); }
    if order.total == 0 { return Err(Error::ZeroTotal); }

    // main logic at top level
    Ok(())
}
```

---

## Newtype Pattern for Domain Types

Use newtypes to make domain concepts distinct at the type level (equivalent to branded types):

```rust
struct UserId(String);
struct PaymentAmount(u64);

fn process_payment(user_id: UserId, amount: PaymentAmount) { ... }

// ❌ Can't accidentally swap arguments
process_payment(PaymentAmount(100), UserId("user-123".into())); // compile error

// ✅ Must construct the correct type
process_payment(UserId("user-123".into()), PaymentAmount(100));
```

---

## Summary Checklist

When writing functional code, verify:

- [ ] No `mut` bindings unless truly necessary
- [ ] Pure functions wherever possible (no side effects in core logic)
- [ ] Code is self-documenting (no explanatory comments needed)
- [ ] Iterator chains (`.map()`, `.filter()`, `.fold()`) over loops
- [ ] Config structs for 3+ parameters
- [ ] Composed small functions, not complex monoliths
- [ ] `Result<T, E>` / `Option<T>` for error handling — no panics in library code
- [ ] Early returns (guard clauses) instead of deep nesting
- [ ] Newtype pattern for domain-meaningful primitives
