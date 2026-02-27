# Architectural Backlog
- Restructure modules by separating `mod.rs` into entity/dto/handlers/router files for better scalability
- Normalize JSON extractor errors into ApiError (avoid raw axum rejection messages)
- Introduce validation layer with field-level error responses