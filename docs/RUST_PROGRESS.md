# Rust Learning Progress

_Tracking Rust concepts learned through building Rift_

---

## Phase 1: Toolchain & Project Structure

| Task                                     | Status | Notes                                                                               |
| ---------------------------------------- | ------ | ----------------------------------------------------------------------------------- |
| Install Rust toolchain (rustup, cargo)   | done   | rustup + cargo installed                                                            |
| Create Cargo workspace (root Cargo.toml) | done   | resolver 3, edition 2024                                                            |
| Create `rift-core` library crate         | done   | library crate with lib.rs                                                           |
| Create `rift-server` binary crate        | done   | renamed from `server` to `rift-server`                                              |
| Create `rift-cli` binary crate           | done   |                                                                                     |
| `cargo build` compiles all three         | done   | path deps wired up between crates                                                   |
| Understand workspace vs crate vs package | done   | workspace = group of crates, crate = compilation unit, package = crate + Cargo.toml |

**Rust concepts:** Cargo.toml, `[workspace]`, `[package]`, `[dependencies]`, edition, resolver, `main.rs` vs `lib.rs`, `cargo build` / `cargo run -p`

---

## Phase 2: Ownership, Borrowing, Lifetimes

| Task                                                          | Status | Notes                                                                    |
| ------------------------------------------------------------- | ------ | ------------------------------------------------------------------------ |
| Define core types in `rift-core` (RevisionId, ChangeId, etc.) | done   | newtype pattern with tuple structs                                       |
| Pass types between functions without cloning everything       | done   | `describe(&revision)` borrows, doesn't move                              |
| Understand why the compiler rejects your first borrow         | done   | passing by value moves ownership; `&` borrows without transferring       |
| Use `&str` vs `String` correctly                              | done   | using `String` for owned data in structs, `&` for borrowing in functions |

**Rust concepts:** ownership, move semantics, `&` references, `&mut`, lifetimes (`'a`), `String` vs `&str`, `Clone` vs `Copy`

---

## Phase 3: Structs, Enums, Pattern Matching

| Task                                                | Status | Notes |
| --------------------------------------------------- | ------ | ----- |
| Model Revision, Change, Stack as structs            |        |       |
| Model StackStatus as an enum (Open, Merged, Closed) |        |       |
| Model BlockedReason as an Option<enum>              |        |       |
| Use pattern matching for stack state transitions    |        |       |
| Handle `Option` and `Result` without unwrap         |        |       |

**Rust concepts:** `struct`, `enum`, `match`, `Option<T>`, `Result<T, E>`, `if let`, destructuring, newtype pattern

---

## Phase 4: Traits & Generics

| Task                                     | Status | Notes |
| ---------------------------------------- | ------ | ----- |
| Implement `Display` for core types       |        |       |
| Define a trait for storage operations    |        |       |
| Understand trait bounds and `impl Trait` |        |       |

**Rust concepts:** `trait`, `impl`, `where` clauses, `dyn Trait` vs `impl Trait`, `derive` macros, trait objects

---

## Phase 5: Error Handling

| Task                                                            | Status | Notes |
| --------------------------------------------------------------- | ------ | ----- |
| Define Rift error types with `thiserror`                        |        |       |
| Map spec error codes (409 change_conflict, etc.) to error types |        |       |
| Use `anyhow` in binary crates, `thiserror` in library           |        |       |
| Replace all `.unwrap()` with proper error handling              |        |       |

**Rust concepts:** `Result`, `?` operator, `From` trait for error conversion, `thiserror`, `anyhow`, error propagation

---

## Phase 6: Async Rust

| Task                                         | Status | Notes |
| -------------------------------------------- | ------ | ----- |
| Set up tokio runtime in `rift-server`        |        |       |
| Write first async function                   |        |       |
| Understand `Future`, `.await`, `Send` bounds |        |       |

**Rust concepts:** `async fn`, `.await`, `tokio`, `Future` trait, `Send + Sync`, pinning, async closures

---

## Phase 7: Axum (HTTP Server)

| Task                                                                   | Status | Notes |
| ---------------------------------------------------------------------- | ------ | ----- |
| Hello world Axum server on port 8080                                   |        |       |
| Add health check route (`GET /health`)                                 |        |       |
| Add JSON request/response (`serde_json`)                               |        |       |
| Implement auth middleware (token validation)                           |        |       |
| Implement repo routes (`POST /v1/repos`, `GET /v1/repos/:owner/:name`) |        |       |
| Implement push routes (validate + push revisions)                      |        |       |
| Implement stack routes (create, list, detail, merge)                   |        |       |
| Implement review routes (submit review, add reviewers)                 |        |       |
| Implement comment routes (create, list)                                |        |       |
| Implement diff/interdiff routes                                        |        |       |
| Implement Git smart HTTP (upload-pack)                                 |        |       |

**Rust concepts:** Axum router, extractors (`Path`, `Query`, `Json`), state management, middleware, `tower` layers, response types

---

## Phase 8: Database (sqlx + Postgres)

| Task                                                   | Status | Notes |
| ------------------------------------------------------ | ------ | ----- |
| Connect to Postgres with sqlx                          |        |       |
| Write and run migrations (schema from spec section 12) |        |       |
| Query with compile-time checked SQL                    |        |       |
| Implement cursor-based pagination                      |        |       |

**Rust concepts:** `sqlx`, connection pools, `query!` vs `query_as!`, migrations, `FromRow`, transactions

---

## Phase 9: Testing

| Task                                     | Status | Notes |
| ---------------------------------------- | ------ | ----- |
| Unit tests for core types and logic      |        |       |
| Integration tests for API routes         |        |       |
| Test push/validate/merge flow end-to-end |        |       |

**Rust concepts:** `#[cfg(test)]`, `#[test]`, `#[tokio::test]`, test modules, `assert!`, `assert_eq!`, test fixtures

---

## Phase 10: Serde & API Contracts

| Task                                                       | Status | Notes |
| ---------------------------------------------------------- | ------ | ----- |
| Derive `Serialize`/`Deserialize` on all API types          |        |       |
| Match spec error response format (section A.3)             |        |       |
| Handle pagination response shape (`items` + `next_cursor`) |        |       |

**Rust concepts:** `serde`, `#[serde(rename_all)]`, `#[serde(skip)]`, custom serializers, `serde_json`

---

## Current Focus

**Phase 3 — Structs, Enums, Pattern Matching.** Build StackStatus, BlockedReason, Role enums and Stack struct. Implement `can_merge` using pattern matching.
