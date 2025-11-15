use async_graphql::{InputObject, SimpleObject};

use crate::domain::models::{Organization as OrgModel, Team as TeamModel};

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

// -------- Inputs --------

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
