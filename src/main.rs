use anyhow::Result;
use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Router, extract::State, http::HeaderMap, routing::post};
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

use paastel::graphql::mutation::MutationRoot;
use paastel::graphql::query::QueryRoot;
use paastel::graphql::state::AppState;

type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    let pool = PgPool::connect(&database_url).await?;
    let state = AppState { pool };

    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(state.clone())
        .finish();

    let app = Router::new()
        .route("/graphql", post(graphql_handler).get(graphiql))
        .with_state(schema);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}

async fn graphql_handler(
    State(schema): State<AppSchema>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();
    request = request.data(headers);
    schema.execute(request).await.into()
}

/// Simple GraphiQL-like playground using async-graphql built-in HTML.
async fn graphiql() -> axum::response::Html<String> {
    use async_graphql::http::GraphiQLSource;

    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
