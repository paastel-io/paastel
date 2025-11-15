use async_graphql::{Context, Object, Result as GqlResult};

use crate::domain::models::{NewOrganization, NewTeam};
use crate::graphql::state::AppState;
use crate::graphql::types::{
    CreateOrganizationInput, CreateTeamInput, OrganizationGql, TeamGql,
};
use crate::infrastructure::repositories::{
    OrganizationRepository, TeamRepository,
};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new organization.
    async fn create_organization(
        &self,
        ctx: &Context<'_>,
        input: CreateOrganizationInput,
    ) -> GqlResult<OrganizationGql> {
        let state = ctx.data::<AppState>()?;
        let repo = OrganizationRepository::new(state.pool.clone());

        let new_org = NewOrganization {
            name: input.name,
            slug: input.slug,
            description: input.description,
        };

        let org = repo
            .create(new_org)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(org.into())
    }

    /// Create a new team inside an organization.
    async fn create_team(
        &self,
        ctx: &Context<'_>,
        input: CreateTeamInput,
    ) -> GqlResult<TeamGql> {
        let state = ctx.data::<AppState>()?;
        let repo = TeamRepository::new(state.pool.clone());

        // Here you could check if the organization exists or if the user has permissions.
        let new_team = NewTeam {
            organization_id: input.organization_id,
            name: input.name,
            slug: input.slug,
            description: input.description,
        };

        let team = repo
            .create(new_team)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(team.into())
    }
}
