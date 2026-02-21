# Axum API Playground

## Why this repository exists

This repository is my personal playground for learning and mastering backend development in **Rust**, starting with the **Axum** web framework.

As a software engineer primarily experienced in frontend development, I’m building this project to:

- Strengthen my backend fundamentals
- Deepen my understanding of Rust’s async model
- Explore modern Rust web tooling (Axum, Tokio, Serde)
- Practice clean API design and structured project setup
- Build toward production-ready Rust services

---

## Current Scope

Initial minimal API setup:

- `GET /health` → basic health check
- `GET /json` → simple JSON response

### Tech stack

- **Axum**
- **Tokio** (async runtime)
- **Serde** (serialization)

---

## Goals

This repository will progressively evolve to include:

- Request extractors
- Shared application state
- Middleware (logging, CORS)
- Error handling patterns
- Structured logging (tracing)
- Project architecture improvements
- Possibly database integration

The objective is not just to “make it work”, but to understand the design philosophy behind Rust web services and write idiomatic, maintainable code.

---

## Run locally

```bash
cargo run
```

Then visit
- http://127.0.0.1:3000/health
- http://127.0.0.1:3000/json