use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::time::OffsetDateTime;

// ---------- Enums mapeando ENUMs do Postgres ----------

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "org_role", rename_all = "lowercase")]
pub enum OrgRole {
    Owner,
    Admin,
    Member,
    Billing,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "team_role", rename_all = "lowercase")]
pub enum TeamRole {
    Member,
    Maintainer,
    Lead,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "app_role", rename_all = "lowercase")]
pub enum AppRole {
    Owner,
    Maintainer,
    Deployer,
    Viewer,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "release_status", rename_all = "lowercase")]
pub enum ReleaseStatus {
    Pending,
    Built,
    Failed,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "deploy_status", rename_all = "lowercase")]
pub enum DeployStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "build_status", rename_all = "lowercase")]
pub enum BuildStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Canceled,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "build_trigger", rename_all = "lowercase")]
pub enum BuildTrigger {
    Manual,
    GitPush,
    Api,
}

// ---------- Organizations ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrganization {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

// ---------- Users ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub last_login_at: Option<OffsetDateTime>,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

// ---------- Organization memberships ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationMembership {
    pub organization_id: i64,
    pub user_id: i64,
    pub role: OrgRole,
    pub created_at: OffsetDateTime,
}

// ---------- Teams ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Team {
    pub id: i64,
    pub organization_id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTeam {
    pub organization_id: i64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

// ---------- Team memberships ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamMembership {
    pub team_id: i64,
    pub user_id: i64,
    pub role: TeamRole,
    pub created_at: OffsetDateTime,
}

// ---------- Apps ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct App {
    pub id: i64,
    pub organization_id: i64,
    pub team_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub repo_url: Option<String>,
    pub created_by: Option<i64>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub deleted_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewApp {
    pub organization_id: i64,
    pub team_id: Option<i64>,
    pub name: String,
    pub slug: String,
    pub repo_url: Option<String>,
    pub created_by: Option<i64>,
}

// ---------- App memberships ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppMembership {
    pub app_id: i64,
    pub user_id: i64,
    pub role: AppRole,
    pub created_at: OffsetDateTime,
}

// ---------- App secrets ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppSecret {
    pub id: i64,
    pub app_id: i64,
    pub environment: String,
    pub key: String,
    pub value: String,
    pub created_by: Option<i64>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAppSecret {
    pub app_id: i64,
    pub environment: String,
    pub key: String,
    pub value: String,
    pub created_by: Option<i64>,
}

// ---------- Releases ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Release {
    pub id: i64,
    pub app_id: i64,
    pub version: String,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub image_ref: Option<String>,
    pub status: ReleaseStatus,
    pub created_by: Option<i64>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewRelease {
    pub app_id: i64,
    pub version: String,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub image_ref: Option<String>,
    pub created_by: Option<i64>,
    pub changelog: Option<String>,
}

// ---------- Deploys ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Deploy {
    pub id: i64,
    pub app_id: i64,
    pub release_id: i64,
    pub environment: String,
    pub status: DeployStatus,
    pub triggered_by: Option<i64>,
    pub target_cluster: Option<String>,
    pub target_region: Option<String>,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub pipeline_url: Option<String>,
    pub logs_url: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewDeploy {
    pub app_id: i64,
    pub release_id: i64,
    pub environment: String,
    pub status: DeployStatus,
    pub triggered_by: Option<i64>,
    pub target_cluster: Option<String>,
    pub target_region: Option<String>,
    pub pipeline_url: Option<String>,
    pub logs_url: Option<String>,
    pub error_message: Option<String>,
}

// ---------- Build jobs ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BuildJob {
    pub id: i64,
    pub app_id: i64,
    pub release_id: Option<i64>,
    pub status: BuildStatus,
    pub trigger: BuildTrigger,
    pub triggered_by: Option<i64>,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub image_ref: Option<String>,
    pub runner_name: Option<String>,
    pub runner_type: Option<String>,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub logs_url: Option<String>,
    pub pipeline_url: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBuildJob {
    pub app_id: i64,
    pub release_id: Option<i64>,
    pub trigger: BuildTrigger,
    pub triggered_by: Option<i64>,
    pub commit_sha: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    pub image_ref: Option<String>,
    pub runner_name: Option<String>,
    pub runner_type: Option<String>,
    pub logs_url: Option<String>,
    pub pipeline_url: Option<String>,
    pub error_message: Option<String>,
}

// ---------- Build steps ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BuildStep {
    pub id: i64,
    pub build_id: i64,
    pub position: i32,
    pub name: String,
    pub status: BuildStatus,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub logs_url: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBuildStep {
    pub build_id: i64,
    pub position: i32,
    pub name: String,
    pub status: BuildStatus,
    pub logs_url: Option<String>,
    pub error_message: Option<String>,
}

// ---------- Build logs ----------

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BuildLog {
    pub id: i64,
    pub build_id: i64,
    pub step_id: Option<i64>,
    pub chunk_index: i32,
    pub content: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBuildLog {
    pub build_id: i64,
    pub step_id: Option<i64>,
    pub chunk_index: i32,
    pub content: String,
}
