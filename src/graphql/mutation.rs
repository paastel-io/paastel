use async_graphql::{Context, Object, Result as GqlResult};
use rand::RngCore;

use crate::domain::models::{NewAuthToken, NewOrganization, NewTeam, NewUser};
use crate::graphql::auth_helpers::get_current_user;
use crate::graphql::state::AppState;
use crate::graphql::types::{
    AccessTokenGql, CreateOrganizationInput, CreateTeamInput, OrganizationGql,
    RegisterUserInput, RegisterUserPayload, TeamGql,
};
use crate::infrastructure::repositories::{
    AuthTokenRepository, OrganizationRepository, TeamRepository,
    UserRepository,
};

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Register a new user and return a personal access token for CLI usage.
    ///
    /// This mutation is unauthenticated (bootstrap). In a real system you may
    /// want to restrict or protect this in some way.
    async fn register_user(
        &self,
        ctx: &Context<'_>,
        input: RegisterUserInput,
    ) -> GqlResult<RegisterUserPayload> {
        let state = ctx.data::<AppState>()?;

        let user_repo = UserRepository::new(state.pool.clone());
        let token_repo = AuthTokenRepository::new(state.pool.clone());

        // TODO: hash password properly (argon2, bcrypt, etc.)
        let new_user = NewUser {
            name: input.name,
            email: input.email,
            password_hash: input.password, // placeholder
        };

        let user = user_repo
            .create(new_user)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        // generate random token (32 bytes hex)
        let token_string = generate_token_string();

        let new_token = NewAuthToken {
            user_id: user.id,
            token: token_string.clone(),
            description: Some("CLI default token".to_string()),
        };

        token_repo
            .create(new_token)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(RegisterUserPayload {
            user: user.into(),
            token: AccessTokenGql {
                token: token_string,
                description: Some("CLI default token".to_string()),
            },
        })
    }

    /// Create a new organization.
    async fn create_organization(
        &self,
        ctx: &Context<'_>,
        input: CreateOrganizationInput,
    ) -> GqlResult<OrganizationGql> {
        // ensure we have a valid user
        let _current = get_current_user(ctx).await?;

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
        // ensure we have a valid user
        let _current = get_current_user(ctx).await?;

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

fn generate_token_string() -> String {
    // 32 random bytes -> hex string (64 chars)
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
