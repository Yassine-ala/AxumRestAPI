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

The project has evolved beyond a simple CRUD API and is now structured around a **metadata-driven architecture** designed to support dynamic form generation.

Instead of hardcoding fields at the application layer, the system models form structure and behavior at the database level through four core entities.

---

## Core Entities

### 1. Domains

A domain represents an isolated configuration context (for example: a country, organization, or environment).  
Each domain defines how data should behave within its scope.

---

### 2. Patients

Represents the primary business entity (subject to evolve into a more generic “Identity” model).  
Patients belong to a domain and are the records that will eventually store dynamic attribute values.

---

### 3. Attributes (Global)

Attributes define the structure of data fields globally.  
They describe **what a field is**, not how it behaves.

An attribute includes:

- A stable `key`
- A display `label`
- A `data_type`
- An optional `description`
- `created_at` and `updated_at` timestamps

These attributes act as reusable building blocks across multiple domains.

---

### 4. Attribute Configurations (Domain-Scoped)

Attribute configurations define **how a given attribute behaves inside a specific domain**.

For each `(domain, attribute)` pair, the configuration controls:

- Whether the attribute is mandatory in forms
- Whether it appears in search
- Whether it appears in banners
- Its search weight (ranking importance)
- Its search index (ordering priority)

This separation between **definition (Attribute)** and **behavior (Attribute Configuration)** enables:

- Domain-specific form rules
- Dynamic UI generation
- Search prioritization strategies
- Future extensibility without schema changes

---

## Architectural Direction

The long-term goal of this structure is to enable:

- Dynamic form construction based on domain configuration
- Flexible search behavior driven by metadata
- Clean separation between data definition and domain-specific rules
- Strong control over SQL and transactional behavior

The API is intentionally implemented using:

- Explicit SQL queries via `sqlx`
- Transaction handling for domain-scoped updates
- Replace-all strategy for dependent collections
- Structured error mapping
- Docker-managed local infrastructure

The focus remains on mastering backend fundamentals rather than relying on heavy abstractions.

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

