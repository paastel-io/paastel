use anyhow::Result;
use sqlx::{PgPool, query_as};

use crate::domain::models::*;

// ---------- OrganizationRepository ----------

#[derive(Clone)]
pub struct OrganizationRepository {
    pool: PgPool,
}

impl OrganizationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Organization>> {
        let org = query_as::<_, Organization>(
            "SELECT * FROM organizations WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(org)
    }

    pub async fn find_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Organization>> {
        let org = query_as::<_, Organization>(
            "SELECT * FROM organizations WHERE slug = $1 AND deleted_at IS NULL",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(org)
    }

    pub async fn create(
        &self,
        new_org: NewOrganization,
    ) -> Result<Organization> {
        let org = query_as::<_, Organization>(
            r#"
            INSERT INTO organizations (name, slug, description)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(new_org.name)
        .bind(new_org.slug)
        .bind(new_org.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }
}

// ---------- UserRepository ----------

#[derive(Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<User>> {
        let user = query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn create(&self, new_user: NewUser) -> Result<User> {
        let user = query_as::<_, User>(
            r#"
            INSERT INTO users (name, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(new_user.name)
        .bind(new_user.email)
        .bind(new_user.password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }
}

// ---------- OrganizationMembershipRepository ----------

#[derive(Clone)]
pub struct OrganizationMembershipRepository {
    pool: PgPool,
}

impl OrganizationMembershipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_organization(
        &self,
        organization_id: i64,
    ) -> Result<Vec<OrganizationMembership>> {
        let rows = query_as::<_, OrganizationMembership>(
            r#"
            SELECT * FROM organization_memberships
            WHERE organization_id = $1
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_by_user(
        &self,
        user_id: i64,
    ) -> Result<Vec<OrganizationMembership>> {
        let rows = query_as::<_, OrganizationMembership>(
            r#"
            SELECT * FROM organization_memberships
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn upsert_membership(
        &self,
        organization_id: i64,
        user_id: i64,
        role: OrgRole,
    ) -> Result<OrganizationMembership> {
        let row = query_as::<_, OrganizationMembership>(
            r#"
            INSERT INTO organization_memberships (organization_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (organization_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            RETURNING *
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_membership(
        &self,
        organization_id: i64,
        user_id: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM organization_memberships
            WHERE organization_id = $1 AND user_id = $2
            "#,
        )
        .bind(organization_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ---------- TeamRepository ----------

#[derive(Clone)]
pub struct TeamRepository {
    pool: PgPool,
}

impl TeamRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Team>> {
        let team = query_as::<_, Team>(
            "SELECT * FROM teams WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(team)
    }

    pub async fn list_by_organization(
        &self,
        organization_id: i64,
    ) -> Result<Vec<Team>> {
        let teams = query_as::<_, Team>(
            r#"
            SELECT * FROM teams
            WHERE organization_id = $1 AND deleted_at IS NULL
            ORDER BY name
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(teams)
    }

    pub async fn create(&self, new_team: NewTeam) -> Result<Team> {
        let team = query_as::<_, Team>(
            r#"
            INSERT INTO teams (organization_id, name, slug, description)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(new_team.organization_id)
        .bind(new_team.name)
        .bind(new_team.slug)
        .bind(new_team.description)
        .fetch_one(&self.pool)
        .await?;

        Ok(team)
    }
}

// ---------- TeamMembershipRepository ----------

#[derive(Clone)]
pub struct TeamMembershipRepository {
    pool: PgPool,
}

impl TeamMembershipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_team(
        &self,
        team_id: i64,
    ) -> Result<Vec<TeamMembership>> {
        let rows = query_as::<_, TeamMembership>(
            r#"
            SELECT * FROM team_memberships
            WHERE team_id = $1
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_by_user(
        &self,
        user_id: i64,
    ) -> Result<Vec<TeamMembership>> {
        let rows = query_as::<_, TeamMembership>(
            r#"
            SELECT * FROM team_memberships
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn upsert_membership(
        &self,
        team_id: i64,
        user_id: i64,
        role: TeamRole,
    ) -> Result<TeamMembership> {
        let row = query_as::<_, TeamMembership>(
            r#"
            INSERT INTO team_memberships (team_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (team_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            RETURNING *
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_membership(
        &self,
        team_id: i64,
        user_id: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM team_memberships
            WHERE team_id = $1 AND user_id = $2
            "#,
        )
        .bind(team_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ---------- AppRepository ----------

#[derive(Clone)]
pub struct AppRepository {
    pool: PgPool,
}

impl AppRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<App>> {
        let app = query_as::<_, App>(
            "SELECT * FROM apps WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(app)
    }

    pub async fn find_by_slug(
        &self,
        organization_id: i64,
        slug: &str,
    ) -> Result<Option<App>> {
        let app = query_as::<_, App>(
            r#"
            SELECT * FROM apps
            WHERE organization_id = $1
              AND slug = $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(organization_id)
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;

        Ok(app)
    }

    pub async fn list_by_organization(
        &self,
        organization_id: i64,
    ) -> Result<Vec<App>> {
        let apps = query_as::<_, App>(
            r#"
            SELECT * FROM apps
            WHERE organization_id = $1
              AND deleted_at IS NULL
            ORDER BY name
            "#,
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(apps)
    }

    pub async fn list_by_team(&self, team_id: i64) -> Result<Vec<App>> {
        let apps = query_as::<_, App>(
            r#"
            SELECT * FROM apps
            WHERE team_id = $1
              AND deleted_at IS NULL
            ORDER BY name
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(apps)
    }

    pub async fn create(&self, new_app: NewApp) -> Result<App> {
        let app = query_as::<_, App>(
            r#"
            INSERT INTO apps (organization_id, team_id, name, slug, repo_url, created_by)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(new_app.organization_id)
        .bind(new_app.team_id)
        .bind(new_app.name)
        .bind(new_app.slug)
        .bind(new_app.repo_url)
        .bind(new_app.created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(app)
    }
}

// ---------- AppMembershipRepository ----------

#[derive(Clone)]
pub struct AppMembershipRepository {
    pool: PgPool,
}

impl AppMembershipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_app(
        &self,
        app_id: i64,
    ) -> Result<Vec<AppMembership>> {
        let rows = query_as::<_, AppMembership>(
            r#"
            SELECT * FROM app_memberships
            WHERE app_id = $1
            "#,
        )
        .bind(app_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_by_user(
        &self,
        user_id: i64,
    ) -> Result<Vec<AppMembership>> {
        let rows = query_as::<_, AppMembership>(
            r#"
            SELECT * FROM app_memberships
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn upsert_membership(
        &self,
        app_id: i64,
        user_id: i64,
        role: AppRole,
    ) -> Result<AppMembership> {
        let row = query_as::<_, AppMembership>(
            r#"
            INSERT INTO app_memberships (app_id, user_id, role)
            VALUES ($1, $2, $3)
            ON CONFLICT (app_id, user_id)
            DO UPDATE SET role = EXCLUDED.role
            RETURNING *
            "#,
        )
        .bind(app_id)
        .bind(user_id)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_membership(
        &self,
        app_id: i64,
        user_id: i64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM app_memberships
            WHERE app_id = $1 AND user_id = $2
            "#,
        )
        .bind(app_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ---------- AppSecretRepository ----------

#[derive(Clone)]
pub struct AppSecretRepository {
    pool: PgPool,
}

impl AppSecretRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_app_env(
        &self,
        app_id: i64,
        environment: &str,
    ) -> Result<Vec<AppSecret>> {
        let rows = query_as::<_, AppSecret>(
            r#"
            SELECT * FROM app_secrets
            WHERE app_id = $1
              AND environment = $2
            ORDER BY key
            "#,
        )
        .bind(app_id)
        .bind(environment)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn upsert_secret(
        &self,
        new_secret: NewAppSecret,
    ) -> Result<AppSecret> {
        let row = query_as::<_, AppSecret>(
            r#"
            INSERT INTO app_secrets (app_id, environment, key, value, created_by)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (app_id, environment, key)
            DO UPDATE SET
                value = EXCLUDED.value,
                updated_at = NOW(),
                created_by = EXCLUDED.created_by
            RETURNING *
            "#,
        )
        .bind(new_secret.app_id)
        .bind(new_secret.environment)
        .bind(new_secret.key)
        .bind(new_secret.value)
        .bind(new_secret.created_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn delete_secret(
        &self,
        app_id: i64,
        environment: &str,
        key: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM app_secrets
            WHERE app_id = $1
              AND environment = $2
              AND key = $3
            "#,
        )
        .bind(app_id)
        .bind(environment)
        .bind(key)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ---------- ReleaseRepository ----------

#[derive(Clone)]
pub struct ReleaseRepository {
    pool: PgPool,
}

impl ReleaseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Release>> {
        let row =
            query_as::<_, Release>("SELECT * FROM releases WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row)
    }

    pub async fn find_by_app_version(
        &self,
        app_id: i64,
        version: &str,
    ) -> Result<Option<Release>> {
        let row = query_as::<_, Release>(
            r#"
            SELECT * FROM releases
            WHERE app_id = $1 AND version = $2
            "#,
        )
        .bind(app_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_by_app(&self, app_id: i64) -> Result<Vec<Release>> {
        let rows = query_as::<_, Release>(
            r#"
            SELECT * FROM releases
            WHERE app_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(app_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create(&self, new_release: NewRelease) -> Result<Release> {
        let row = query_as::<_, Release>(
            r#"
            INSERT INTO releases (
                app_id, version, commit_sha, branch, tag, image_ref,
                status, created_by, changelog
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8)
            RETURNING *
            "#,
        )
        .bind(new_release.app_id)
        .bind(new_release.version)
        .bind(new_release.commit_sha)
        .bind(new_release.branch)
        .bind(new_release.tag)
        .bind(new_release.image_ref)
        .bind(new_release.created_by)
        .bind(new_release.changelog)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }
}

// ---------- DeployRepository ----------

#[derive(Clone)]
pub struct DeployRepository {
    pool: PgPool,
}

impl DeployRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Deploy>> {
        let row = query_as::<_, Deploy>("SELECT * FROM deploys WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row)
    }

    pub async fn list_by_app_env(
        &self,
        app_id: i64,
        environment: &str,
    ) -> Result<Vec<Deploy>> {
        let rows = query_as::<_, Deploy>(
            r#"
            SELECT * FROM deploys
            WHERE app_id = $1 AND environment = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(app_id)
        .bind(environment)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn list_by_release(
        &self,
        release_id: i64,
    ) -> Result<Vec<Deploy>> {
        let rows = query_as::<_, Deploy>(
            r#"
            SELECT * FROM deploys
            WHERE release_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(release_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create(&self, new_deploy: NewDeploy) -> Result<Deploy> {
        let row = query_as::<_, Deploy>(
            r#"
            INSERT INTO deploys (
                app_id, release_id, environment, status,
                triggered_by, target_cluster, target_region,
                pipeline_url, logs_url, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(new_deploy.app_id)
        .bind(new_deploy.release_id)
        .bind(new_deploy.environment)
        .bind(new_deploy.status)
        .bind(new_deploy.triggered_by)
        .bind(new_deploy.target_cluster)
        .bind(new_deploy.target_region)
        .bind(new_deploy.pipeline_url)
        .bind(new_deploy.logs_url)
        .bind(new_deploy.error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }
}

// ---------- BuildJobRepository ----------

#[derive(Clone)]
pub struct BuildJobRepository {
    pool: PgPool,
}

impl BuildJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<BuildJob>> {
        let row =
            query_as::<_, BuildJob>("SELECT * FROM build_jobs WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row)
    }

    pub async fn list_recent_by_app(
        &self,
        app_id: i64,
        limit: i64,
    ) -> Result<Vec<BuildJob>> {
        let rows = query_as::<_, BuildJob>(
            r#"
            SELECT * FROM build_jobs
            WHERE app_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(app_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create(&self, new_job: NewBuildJob) -> Result<BuildJob> {
        let row = query_as::<_, BuildJob>(
            r#"
            INSERT INTO build_jobs (
                app_id, release_id, status, trigger, triggered_by,
                commit_sha, branch, tag, image_ref,
                runner_name, runner_type, logs_url, pipeline_url, error_message
            )
            VALUES (
                $1, $2, 'pending', $3, $4,
                $5, $6, $7, $8,
                $9, $10, $11, $12, $13
            )
            RETURNING *
            "#,
        )
        .bind(new_job.app_id)
        .bind(new_job.release_id)
        .bind(new_job.trigger)
        .bind(new_job.triggered_by)
        .bind(new_job.commit_sha)
        .bind(new_job.branch)
        .bind(new_job.tag)
        .bind(new_job.image_ref)
        .bind(new_job.runner_name)
        .bind(new_job.runner_type)
        .bind(new_job.logs_url)
        .bind(new_job.pipeline_url)
        .bind(new_job.error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }
}

// ---------- BuildStepRepository ----------

#[derive(Clone)]
pub struct BuildStepRepository {
    pool: PgPool,
}

impl BuildStepRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_build(
        &self,
        build_id: i64,
    ) -> Result<Vec<BuildStep>> {
        let rows = query_as::<_, BuildStep>(
            r#"
            SELECT * FROM build_steps
            WHERE build_id = $1
            ORDER BY position
            "#,
        )
        .bind(build_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create(&self, new_step: NewBuildStep) -> Result<BuildStep> {
        let row = query_as::<_, BuildStep>(
            r#"
            INSERT INTO build_steps (
                build_id, position, name, status, logs_url, error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(new_step.build_id)
        .bind(new_step.position)
        .bind(new_step.name)
        .bind(new_step.status)
        .bind(new_step.logs_url)
        .bind(new_step.error_message)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }
}

// ---------- BuildLogRepository ----------

#[derive(Clone)]
pub struct BuildLogRepository {
    pool: PgPool,
}

impl BuildLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_build(&self, build_id: i64) -> Result<Vec<BuildLog>> {
        let rows = query_as::<_, BuildLog>(
            r#"
            SELECT * FROM build_logs
            WHERE build_id = $1
            ORDER BY chunk_index
            "#,
        )
        .bind(build_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn create(&self, new_log: NewBuildLog) -> Result<BuildLog> {
        let row = query_as::<_, BuildLog>(
            r#"
            INSERT INTO build_logs (
                build_id, step_id, chunk_index, content
            )
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(new_log.build_id)
        .bind(new_log.step_id)
        .bind(new_log.chunk_index)
        .bind(new_log.content)
        .fetch_one(&self.pool)
        .await?;

        Ok(row)
    }
}
