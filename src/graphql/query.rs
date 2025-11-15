use async_graphql::{Context, Object, Result as GqlResult};

use crate::graphql::state::AppState;
use crate::graphql::types::{OrganizationGql, TeamGql};
use crate::infrastructure::repositories::{
    OrganizationRepository, TeamRepository,
};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Simple ping query, useful for health checking GraphQL.
    async fn api_version(&self) -> &str {
        "1.0.0"
    }

    async fn organization(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> GqlResult<Option<OrganizationGql>> {
        let state = ctx.data::<AppState>()?;
        let repo = OrganizationRepository::new(state.pool.clone());

        let org = repo
            .find_by_id(id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(org.map(Into::into))
    }

    async fn team(
        &self,
        ctx: &Context<'_>,
        id: i64,
    ) -> GqlResult<Option<TeamGql>> {
        let state = ctx.data::<AppState>()?;
        let repo = TeamRepository::new(state.pool.clone());

        let team = repo
            .find_by_id(id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(team.map(Into::into))
    }
}
