use async_graphql::{InputObject, SimpleObject};

use crate::domain::models::{
    Organization as OrgModel, Team as TeamModel, User,
};

// ------------ User ------------

#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "User")]
pub struct UserGql {
    pub id: i64,
    pub name: String,
    pub email: String,
}

impl From<User> for UserGql {
    fn from(u: User) -> Self {
        Self { id: u.id, name: u.name, email: u.email }
    }
}

// GraphQL Organization exposed type
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "Organization")]
pub struct OrganizationGql {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

impl From<OrgModel> for OrganizationGql {
    fn from(org: OrgModel) -> Self {
        Self {
            id: org.id,
            name: org.name,
            slug: org.slug,
            description: org.description,
        }
    }
}

// GraphQL Team exposed type
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "Team")]
pub struct TeamGql {
    pub id: i64,
    pub organization_id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

impl From<TeamModel> for TeamGql {
    fn from(team: TeamModel) -> Self {
        Self {
            id: team.id,
            organization_id: team.organization_id,
            name: team.name,
            slug: team.slug,
            description: team.description,
        }
    }
}

// ------------ AuthToken (GraphQL) ------------

#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "AccessToken")]
pub struct AccessTokenGql {
    /// Token string that the CLI must store and send in the Authorization header.
    pub token: String,
    pub description: Option<String>,
}

// -------- Inputs --------

#[derive(Debug, InputObject)]
pub struct RegisterUserInput {
    pub name: String,
    pub email: String,
    /// Plain password for now. You should hash it before storing.
    pub password: String,
}

#[derive(Debug, SimpleObject)]
pub struct RegisterUserPayload {
    pub user: UserGql,
    pub token: AccessTokenGql,
}

#[derive(Debug, InputObject)]
pub struct CreateOrganizationInput {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, InputObject)]
pub struct CreateTeamInput {
    /// Organization that owns this team
    pub organization_id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}
