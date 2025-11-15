use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Global CLI configuration stored in ~/.config/paastel/config.toml
#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    #[serde(default)]
    auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AuthConfig {
    /// Full GraphQL endpoint, e.g. "http://localhost:3000/graphql"
    #[serde(default)]
    base_url: String,
    #[serde(default)]
    token: String,
}

/// Session (context) stored in ~/.config/paastel/session.toml
#[derive(Debug, Serialize, Deserialize, Default)]
struct Session {
    #[serde(default)]
    context: SessionContext,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SessionContext {
    #[serde(default)]
    organization_id: Option<i64>,
    #[serde(default)]
    organization_slug: Option<String>,
    #[serde(default)]
    team_id: Option<i64>,
    #[serde(default)]
    team_slug: Option<String>,
}

/// Root CLI
#[derive(Parser, Debug)]
#[command(name = "paastel")]
#[command(about = "PaaStel CLI - manage orgs, teams and apps", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Authentication commands (login, logout, status)
    #[command(subcommand)]
    Auth(AuthCommand),

    /// Organization commands
    #[command(subcommand)]
    Org(OrgCommand),

    /// Team commands
    #[command(subcommand)]
    Team(TeamCommand),

    /// Context/session commands
    #[command(subcommand)]
    Context(ContextCommand),

    /// Application commands
    #[command(subcommand)]
    App(AppCommand),
}

#[derive(Subcommand, Debug)]
enum AuthCommand {
    /// Register a new user (bootstrap) and store the token locally
    ///
    /// This calls the GraphQL mutation `registerUser` and saves the
    /// returned access token in config.toml.
    Login {
        /// User name
        #[arg(long)]
        name: Option<String>,
        /// Email used to register
        #[arg(long)]
        email: Option<String>,
        /// Password used to register
        #[arg(long)]
        password: Option<String>,
        /// GraphQL endpoint (override default)
        #[arg(long)]
        base_url: Option<String>,
    },
    /// Remove local authentication
    Logout,
    /// Show current authentication status
    Status,
}

#[derive(Subcommand, Debug)]
enum OrgCommand {
    /// Create a new organization (requires authentication)
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        slug: String,
        #[arg(long)]
        description: Option<String>,
    },
    /// Set current organization in the local session
    Use {
        /// Organization ID
        #[arg(long)]
        id: Option<i64>,
        /// Organization slug
        #[arg(long)]
        slug: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum TeamCommand {
    /// Create a new team in the current organization (requires auth + org)
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        slug: String,
        #[arg(long)]
        description: Option<String>,
    },
    /// Set current team in the local session
    Use {
        /// Team ID
        #[arg(long)]
        id: Option<i64>,
        /// Team slug
        #[arg(long)]
        slug: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ContextCommand {
    /// Show current context (org + team)
    Show,
    /// Clear local session (does not logout)
    Clear,
}

#[derive(Subcommand, Debug)]
enum AppCommand {
    /// Create a new application in the current org/team (requires auth + org + team)
    ///
    /// NOTE: Not implemented in the GraphQL schema yet. This command
    /// will currently return an error.
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        slug: String,
        /// Optional runtime (example: nodejs-22)
        #[arg(long)]
        runtime: Option<String>,
    },
}

// ---------------------------
// Helpers for config/session
// ---------------------------

fn paastel_config_dir() -> Result<PathBuf> {
    let base =
        dirs::config_dir().context("Could not determine config directory")?;
    Ok(base.join("paastel"))
}

fn config_path() -> Result<PathBuf> {
    Ok(paastel_config_dir()?.join("config.toml"))
}

fn session_path() -> Result<PathBuf> {
    Ok(paastel_config_dir()?.join("session.toml"))
}

fn load_config() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }

    let data = fs::read_to_string(&path).with_context(|| {
        format!("Failed to read config file at {}", path.display())
    })?;
    let cfg: Config =
        toml::from_str(&data).context("Failed to parse config.toml")?;
    Ok(cfg)
}

fn save_config(cfg: &Config) -> Result<()> {
    let dir = paastel_config_dir()?;
    fs::create_dir_all(&dir).with_context(|| {
        format!("Failed to create config dir at {}", dir.display())
    })?;

    let path = config_path()?;
    let data =
        toml::to_string_pretty(cfg).context("Failed to serialize config")?;
    fs::write(&path, data).with_context(|| {
        format!("Failed to write config file at {}", path.display())
    })?;
    Ok(())
}

fn load_session() -> Result<Session> {
    let path = session_path()?;
    if !path.exists() {
        return Ok(Session::default());
    }

    let data = fs::read_to_string(&path).with_context(|| {
        format!("Failed to read session file at {}", path.display())
    })?;
    let sess: Session =
        toml::from_str(&data).context("Failed to parse session.toml")?;
    Ok(sess)
}

fn save_session(sess: &Session) -> Result<()> {
    let dir = paastel_config_dir()?;
    fs::create_dir_all(&dir).with_context(|| {
        format!("Failed to create config dir at {}", dir.display())
    })?;

    let path = session_path()?;
    let data =
        toml::to_string_pretty(sess).context("Failed to serialize session")?;
    fs::write(&path, data).with_context(|| {
        format!("Failed to write session file at {}", path.display())
    })?;
    Ok(())
}

// -------------
// GraphQL types
// -------------

#[derive(Debug, Serialize)]
struct GqlRequest<V> {
    query: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<V>,
}

#[derive(Debug, Deserialize)]
struct GqlResponse<D> {
    data: Option<D>,
    errors: Option<Vec<GqlError>>,
}

#[derive(Debug, Deserialize)]
struct GqlError {
    message: String,
    // You can extend with locations, path, extensions, etc.
}

// ---- registerUser ----

#[derive(Debug, Serialize)]
struct RegisterUserVariables<'a> {
    input: RegisterUserInput<'a>,
}

#[derive(Debug, Serialize)]
struct RegisterUserInput<'a> {
    name: &'a str,
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Deserialize)]
struct RegisterUserData {
    registerUser: RegisterUserPayload,
}

#[derive(Debug, Deserialize)]
struct RegisterUserPayload {
    user: GqlUser,
    token: AccessToken,
}

#[derive(Debug, Deserialize)]
struct GqlUser {
    id: i32,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct AccessToken {
    token: String,
    description: Option<String>,
}

// ---- createOrganization ----

#[derive(Debug, Serialize)]
struct CreateOrganizationVariables<'a> {
    input: CreateOrganizationInput<'a>,
}

#[derive(Debug, Serialize)]
struct CreateOrganizationInput<'a> {
    name: &'a str,
    slug: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct CreateOrganizationData {
    createOrganization: OrganizationResponse,
}

#[derive(Debug, Deserialize)]
struct OrganizationResponse {
    id: i32,
    name: String,
    slug: String,
    description: Option<String>,
}

// ---- createTeam ----

#[derive(Debug, Serialize)]
struct CreateTeamVariables<'a> {
    input: CreateTeamInput<'a>,
}

#[derive(Debug, Serialize)]
struct CreateTeamInput<'a> {
    organizationId: i32,
    name: &'a str,
    slug: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct CreateTeamData {
    createTeam: TeamResponse,
}

#[derive(Debug, Deserialize)]
struct TeamResponse {
    id: i32,
    organizationId: i32,
    name: String,
    slug: String,
    description: Option<String>,
}

// -----------------
// GraphQL documents
// -----------------

static REGISTER_USER_MUTATION: &str = r#"
mutation RegisterUser($input: RegisterUserInput!) {
  registerUser(input: $input) {
    user {
      id
      name
      email
    }
    token {
      token
      description
    }
  }
}
"#;

static CREATE_ORGANIZATION_MUTATION: &str = r#"
mutation CreateOrganization($input: CreateOrganizationInput!) {
  createOrganization(input: $input) {
    id
    name
    slug
    description
  }
}
"#;

static CREATE_TEAM_MUTATION: &str = r#"
mutation CreateTeam($input: CreateTeamInput!) {
  createTeam(input: $input) {
    id
    organizationId
    name
    slug
    description
  }
}
"#;

// -----------------
// API call helpers
// -----------------

async fn gql_register_user(
    client: &Client,
    base_url: &str,
    name: &str,
    email: &str,
    password: &str,
) -> Result<RegisterUserPayload> {
    let req_body = GqlRequest {
        query: REGISTER_USER_MUTATION,
        variables: Some(RegisterUserVariables {
            input: RegisterUserInput { name, email, password },
        }),
    };

    let res = client
        .post(base_url)
        .json(&req_body)
        .send()
        .await
        .context("Failed to send registerUser GraphQL request")?;

    if !res.status().is_success() {
        anyhow::bail!("registerUser failed with HTTP status {}", res.status());
    }

    let gql: GqlResponse<RegisterUserData> = res
        .json()
        .await
        .context("Failed to parse GraphQL response for registerUser")?;

    if let Some(errors) = gql.errors {
        let msg = errors
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("; ");
        anyhow::bail!("GraphQL error(s): {msg}");
    }

    let data = gql
        .data
        .ok_or_else(|| anyhow::anyhow!("Missing data in GraphQL response"))?;
    Ok(data.registerUser)
}

async fn gql_create_org(
    client: &Client,
    cfg: &Config,
    name: &str,
    slug: &str,
    description: Option<&str>,
) -> Result<OrganizationResponse> {
    let req_body = GqlRequest {
        query: CREATE_ORGANIZATION_MUTATION,
        variables: Some(CreateOrganizationVariables {
            input: CreateOrganizationInput { name, slug, description },
        }),
    };

    let res = client
        .post(&cfg.auth.base_url)
        .bearer_auth(&cfg.auth.token)
        .json(&req_body)
        .send()
        .await
        .context("Failed to send createOrganization GraphQL request")?;

    if !res.status().is_success() {
        anyhow::bail!(
            "createOrganization failed with HTTP status {}",
            res.status()
        );
    }

    let gql: GqlResponse<CreateOrganizationData> = res
        .json()
        .await
        .context("Failed to parse GraphQL response for createOrganization")?;

    if let Some(errors) = gql.errors {
        let msg = errors
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("; ");
        anyhow::bail!("GraphQL error(s): {msg}");
    }

    let data = gql
        .data
        .ok_or_else(|| anyhow::anyhow!("Missing data in GraphQL response"))?;
    Ok(data.createOrganization)
}

async fn gql_create_team(
    client: &Client,
    cfg: &Config,
    org_id: i64,
    name: &str,
    slug: &str,
    description: Option<&str>,
) -> Result<TeamResponse> {
    let req_body = GqlRequest {
        query: CREATE_TEAM_MUTATION,
        variables: Some(CreateTeamVariables {
            input: CreateTeamInput {
                organizationId: org_id as i32,
                name,
                slug,
                description,
            },
        }),
    };

    let res = client
        .post(&cfg.auth.base_url)
        .bearer_auth(&cfg.auth.token)
        .json(&req_body)
        .send()
        .await
        .context("Failed to send createTeam GraphQL request")?;

    if !res.status().is_success() {
        anyhow::bail!("createTeam failed with HTTP status {}", res.status());
    }

    let gql: GqlResponse<CreateTeamData> = res
        .json()
        .await
        .context("Failed to parse GraphQL response for createTeam")?;

    if let Some(errors) = gql.errors {
        let msg = errors
            .into_iter()
            .map(|e| e.message)
            .collect::<Vec<_>>()
            .join("; ");
        anyhow::bail!("GraphQL error(s): {msg}");
    }

    let data = gql
        .data
        .ok_or_else(|| anyhow::anyhow!("Missing data in GraphQL response"))?;
    Ok(data.createTeam)
}

// --------------------
// Command dispatcher
// --------------------

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let http_client = Client::new();

    match cli.command {
        Commands::Auth(cmd) => handle_auth(cmd, &http_client).await?,
        Commands::Org(cmd) => handle_org(cmd, &http_client).await?,
        Commands::Team(cmd) => handle_team(cmd, &http_client).await?,
        Commands::Context(cmd) => handle_context(cmd)?,
        Commands::App(cmd) => handle_app(cmd, &http_client).await?,
    }

    Ok(())
}

// -------------
// Auth handler
// -------------

async fn handle_auth(cmd: AuthCommand, client: &Client) -> Result<()> {
    match cmd {
        AuthCommand::Login { name, email, password, base_url } => {
            let name = match name {
                Some(v) => v,
                None => prompt("Name: ")?,
            };

            let email = match email {
                Some(v) => v,
                None => prompt("Email: ")?,
            };

            let password = match password {
                Some(v) => v,
                None => prompt_password("Password: ")?,
            };

            let mut cfg = load_config().unwrap_or_default();

            if let Some(base) = base_url {
                cfg.auth.base_url = base;
            } else if cfg.auth.base_url.is_empty() {
                // default GraphQL endpoint
                cfg.auth.base_url =
                    "http://localhost:3000/graphql".to_string();
            }

            let payload = gql_register_user(
                client,
                &cfg.auth.base_url,
                &name,
                &email,
                &password,
            )
            .await?;

            cfg.auth.token = payload.token.token;
            save_config(&cfg)?;
            // clear session when logging in/registering
            save_session(&Session::default())?;

            println!(
                "User registered and logged in as {} ({})",
                payload.user.name, payload.user.email
            );
        }
        AuthCommand::Logout => {
            let mut cfg = load_config().unwrap_or_default();
            cfg.auth.token.clear();
            save_config(&cfg)?;
            println!("Logged out. Token removed from config.toml");
        }
        AuthCommand::Status => {
            let cfg = load_config().unwrap_or_default();
            if cfg.auth.token.is_empty() {
                println!("Not authenticated. Run `paastel auth login` first.");
            } else {
                println!("Authenticated.");
                println!("GraphQL endpoint: {}", cfg.auth.base_url);
                println!("Token: present");
            }
        }
    }
    Ok(())
}

// -------------
// Org handler
// -------------

async fn handle_org(cmd: OrgCommand, client: &Client) -> Result<()> {
    match cmd {
        OrgCommand::Create { name, slug, description } => {
            let cfg = ensure_authenticated()?;
            let org = gql_create_org(
                client,
                &cfg,
                &name,
                &slug,
                description.as_deref(),
            )
            .await?;

            println!(
                "Organization created: {} (id: {}, slug: {})",
                org.name, org.id, org.slug
            );

            // set as current context
            let mut sess = load_session().unwrap_or_default();
            sess.context.organization_id = Some(org.id as i64);
            sess.context.organization_slug = Some(org.slug);
            // when we change org, we can reset team
            sess.context.team_id = None;
            sess.context.team_slug = None;
            save_session(&sess)?;
            println!("Organization set as current context.");
        }
        OrgCommand::Use { id, slug } => {
            let cfg = ensure_authenticated()?;
            if cfg.auth.token.is_empty() {
                anyhow::bail!(
                    "You must be authenticated to use an organization."
                );
            }

            let mut sess = load_session().unwrap_or_default();

            match (id, slug) {
                (Some(id), _) => {
                    sess.context.organization_id = Some(id);
                    sess.context.organization_slug = None;
                }
                (None, Some(slug)) => {
                    sess.context.organization_slug = Some(slug);
                    sess.context.organization_id = None;
                }
                _ => {
                    anyhow::bail!("You must provide either --id or --slug.");
                }
            }

            // when org changes, we usually reset team
            sess.context.team_id = None;
            sess.context.team_slug = None;

            save_session(&sess)?;
            println!("Organization context updated.");
        }
    }

    Ok(())
}

// -------------
// Team handler
// -------------

async fn handle_team(cmd: TeamCommand, client: &Client) -> Result<()> {
    match cmd {
        TeamCommand::Create { name, slug, description } => {
            let cfg = ensure_authenticated()?;
            let sess = load_session().unwrap_or_default();

            let org_id = sess.context.organization_id.ok_or_else(|| {
                anyhow::anyhow!(
                    "No organization selected. Use `paastel org use` first."
                )
            })?;

            let team = gql_create_team(
                client,
                &cfg,
                org_id,
                &name,
                &slug,
                description.as_deref(),
            )
            .await?;

            println!(
                "Team created: {} (id: {}, slug: {})",
                team.name, team.id, team.slug
            );

            let mut sess = sess;
            sess.context.team_id = Some(team.id as i64);
            sess.context.team_slug = Some(team.slug);
            save_session(&sess)?;
            println!("Team set as current context.");
        }
        TeamCommand::Use { id, slug } => {
            let _cfg = ensure_authenticated()?;
            let mut sess = load_session().unwrap_or_default();

            if sess.context.organization_id.is_none()
                && sess.context.organization_slug.is_none()
            {
                anyhow::bail!(
                    "No organization selected. Use `paastel org use` first."
                );
            }

            match (id, slug) {
                (Some(id), _) => {
                    sess.context.team_id = Some(id);
                    sess.context.team_slug = None;
                }
                (None, Some(slug)) => {
                    sess.context.team_slug = Some(slug);
                    sess.context.team_id = None;
                }
                _ => {
                    anyhow::bail!("You must provide either --id or --slug.");
                }
            }

            save_session(&sess)?;
            println!("Team context updated.");
        }
    }

    Ok(())
}

// ----------------
// Context handler
// ----------------

fn handle_context(cmd: ContextCommand) -> Result<()> {
    match cmd {
        ContextCommand::Show => {
            let cfg = load_config().unwrap_or_default();
            let sess = load_session().unwrap_or_default();

            println!("Auth:");
            if cfg.auth.token.is_empty() {
                println!("  Status      : not authenticated");
            } else {
                println!("  Status      : authenticated");
                println!("  Endpoint    : {}", cfg.auth.base_url);
            }

            println!();
            println!("Context:");
            match (
                &sess.context.organization_id,
                &sess.context.organization_slug,
            ) {
                (Some(id), Some(slug)) => {
                    println!("  Organization: {} (id: {})", slug, id);
                }
                (Some(id), None) => {
                    println!("  Organization: (id: {})", id);
                }
                (None, Some(slug)) => {
                    println!("  Organization: {} (id: unknown)", slug);
                }
                (None, None) => {
                    println!("  Organization: (not set)");
                }
            };

            match (&sess.context.team_id, &sess.context.team_slug) {
                (Some(id), Some(slug)) => {
                    println!("  Team        : {} (id: {})", slug, id);
                }
                (Some(id), None) => {
                    println!("  Team        : (id: {})", id);
                }
                (None, Some(slug)) => {
                    println!("  Team        : {} (id: unknown)", slug);
                }
                (None, None) => {
                    println!("  Team        : (not set)");
                }
            };
        }
        ContextCommand::Clear => {
            let path = session_path()?;
            if path.exists() {
                fs::remove_file(&path).with_context(|| {
                    format!("Failed to remove session file {}", path.display())
                })?;
                println!("Session cleared.");
            } else {
                println!("Session not found. Nothing to clear.");
            }
        }
    }

    Ok(())
}

// -------------
// App handler
// -------------

async fn handle_app(cmd: AppCommand, _client: &Client) -> Result<()> {
    match cmd {
        AppCommand::Create { .. } => {
            anyhow::bail!(
                "App creation is not implemented in the GraphQL schema yet. \
                 Once you add a createApp mutation, we can wire it here."
            );
        }
    }
}

// -------------------------
// Small utility functions
// -------------------------

fn ensure_authenticated() -> Result<Config> {
    let cfg = load_config().unwrap_or_default();
    if cfg.auth.token.is_empty() {
        anyhow::bail!(
            "You must be authenticated. Run `paastel auth login` first."
        );
    }
    if cfg.auth.base_url.is_empty() {
        anyhow::bail!(
            "GraphQL endpoint is not configured. Set it during login or in config.toml."
        );
    }
    Ok(cfg)
}

fn prompt(label: &str) -> Result<String> {
    use std::io::{self, Write};

    print!("{label}");
    io::stdout().flush().ok();

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).context("Failed to read from stdin")?;
    Ok(buf.trim().to_string())
}

fn prompt_password(label: &str) -> Result<String> {
    // For now, simple prompt. You can switch to rpassword crate if you want hidden input.
    prompt(label)
}
