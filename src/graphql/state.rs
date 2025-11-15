use sqlx::PgPool;

/// Shared application state injected into GraphQL schema.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}
