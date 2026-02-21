# Axum API Playground

## Why this repository exists

This repository is my personal playground for learning and mastering backend development in **Rust**, using **Axum**, **PostgreSQL**, and **sqlx**.

As a software engineer with a strong frontend background, this project is focused on:

- Building real backend fundamentals
- Understanding Rust’s async model in practice
- Writing explicit SQL instead of relying on heavy abstractions
- Designing clean HTTP APIs with proper error handling
- Managing infrastructure locally using Docker

The goal is not just to make things work, but to understand the mechanisms behind them.

---

## Current State

The project currently exposes a simple but complete CRUD API around a `Patient` entity.

### Patient Model

- `id` (UUID)
- `first_name`
- `last_name`
- `birth_date`
- `email` (optional, unique)
- `created_at`
- `updated_at`

### Available Operations

- `POST /patients` → Create a patient
- `GET /patients` → List patients (supports `?search=` query param)
- `GET /patients/{id}` → Retrieve a specific patient
- `PUT /patients/{id}` → Partially update a patient
- `DELETE /patients/{id}` → Delete a patient

The API uses:

- Typed DTOs for input
- Explicit SQL queries via `sqlx`
- Structured error handling mapped to proper HTTP status codes
- PostgreSQL migrations managed through `sqlx-cli`
- Environment-based configuration via `.env` (local development)

All requests are tested through a Postman collection using environment variables and request chaining.

---

## Running Locally

Start the database:

```bash
docker compose up -d
```

cargo run to start the api at:
```bash
http://127.0.0.1:3000
```

