use async_graphql::{Context, Error as GqlError, Result as GqlResult};
use axum::http::{self, header::AUTHORIZATION};

use crate::graphql::auth::CurrentUser;
use crate::graphql::state::AppState;
use crate::infrastructure::repositories::{AuthTokenRepository, UserRepository};

/// Get the currently authenticated user from the Authorization header.
///
/// Expected header: `Authorization: Bearer <token>`
pub async fn get_current_user(ctx: &Context<'_>) -> GqlResult<CurrentUser> {
    // Read raw headers from async-graphql context
    let headers = ctx
        .data_opt::<http::HeaderMap>()
        .ok_or_else(|| GqlError::new("Missing request headers in context"))?;

    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| GqlError::new("Missing Authorization header"))?;

    // Format: "Bearer TOKEN"
    let token_str = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| GqlError::new("Invalid Authorization format"))?;

    let state = ctx.data::<AppState>()?;
    let token_repo = AuthTokenRepository::new(state.pool.clone());
    let user_repo = UserRepository::new(state.pool.clone());

    let token = token_repo
        .find_valid_by_token(token_str)
        .await
        .map_err(|e| GqlError::new(e.to_string()))?
        .ok_or_else(|| GqlError::new("Invalid or revoked token"))?;

    let user = user_repo
        .find_by_id(token.user_id)
        .await
        .map_err(|e| GqlError::new(e.to_string()))?
        .ok_or_else(|| GqlError::new("User not found for token"))?;

    Ok(CurrentUser { user })
}

