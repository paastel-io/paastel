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
    /// Authenticate and store token locally
    Login {
        /// Email used to authenticate
        #[arg(long)]
        email: Option<String>,
        /// Password used to authenticate
        #[arg(long)]
        password: Option<String>,
        /// API base URL (override default)
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
// API DTO stubs
// -------------

#[derive(Debug, Deserialize)]
struct LoginResponse {
    token: String,
    // you can extend with user info
}

#[derive(Debug, Deserialize)]
struct OrganizationResponse {
    id: i64,
    name: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
struct TeamResponse {
    id: i64,
    name: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
struct AppResponse {
    id: i64,
    name: String,
    slug: String,
}

// -----------------
// API call helpers
// -----------------

async fn api_login(
    client: &Client,
    base_url: &str,
    email: &str,
    password: &str,
) -> Result<LoginResponse> {
    // TODO: replace with your real REST/GraphQL call.
    // This is just a stub structure to show where you integrate.

    #[derive(Serialize)]
    struct Payload<'a> {
        email: &'a str,
        password: &'a str,
    }

    let url = format!("{}/auth/login", base_url.trim_end_matches('/'));
    let res = client
        .post(&url)
        .json(&Payload { email, password })
        .send()
        .await
        .context("Failed to send login request")?;

    if !res.status().is_success() {
        anyhow::bail!("Login failed with status {}", res.status());
    }

    let body: LoginResponse =
        res.json().await.context("Failed to parse login response")?;
    Ok(body)
}

async fn api_create_org(
    client: &Client,
    cfg: &Config,
    name: &str,
    slug: &str,
) -> Result<OrganizationResponse> {
    #[derive(Serialize)]
    struct Payload<'a> {
        name: &'a str,
        slug: &'a str,
    }

    let url = format!("{}/orgs", cfg.auth.base_url.trim_end_matches('/'));
    let res = client
        .post(&url)
        .bearer_auth(&cfg.auth.token)
        .json(&Payload { name, slug })
        .send()
        .await
        .context("Failed to send create org request")?;

    if !res.status().is_success() {
        anyhow::bail!("Create org failed with status {}", res.status());
    }

    let body: OrganizationResponse =
        res.json().await.context("Failed to parse org response")?;
    Ok(body)
}

async fn api_create_team(
    client: &Client,
    cfg: &Config,
    org_id: i64,
    name: &str,
    slug: &str,
) -> Result<TeamResponse> {
    #[derive(Serialize)]
    struct Payload<'a> {
        org_id: i64,
        name: &'a str,
        slug: &'a str,
    }

    let url = format!("{}/teams", cfg.auth.base_url.trim_end_matches('/'));
    let res = client
        .post(&url)
        .bearer_auth(&cfg.auth.token)
        .json(&Payload { org_id, name, slug })
        .send()
        .await
        .context("Failed to send create team request")?;

    if !res.status().is_success() {
        anyhow::bail!("Create team failed with status {}", res.status());
    }

    let body: TeamResponse =
        res.json().await.context("Failed to parse team response")?;
    Ok(body)
}

async fn api_create_app(
    client: &Client,
    cfg: &Config,
    org_id: i64,
    team_id: i64,
    name: &str,
    slug: &str,
    runtime: Option<&str>,
) -> Result<AppResponse> {
    #[derive(Serialize)]
    struct Payload<'a> {
        org_id: i64,
        team_id: i64,
        name: &'a str,
        slug: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        runtime: Option<&'a str>,
    }

    let url = format!("{}/apps", cfg.auth.base_url.trim_end_matches('/'));
    let res = client
        .post(&url)
        .bearer_auth(&cfg.auth.token)
        .json(&Payload { org_id, team_id, name, slug, runtime })
        .send()
        .await
        .context("Failed to send create app request")?;

    if !res.status().is_success() {
        anyhow::bail!("Create app failed with status {}", res.status());
    }

    let body: AppResponse =
        res.json().await.context("Failed to parse app response")?;
    Ok(body)
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
        AuthCommand::Login { email, password, base_url } => {
            let email = match email {
                Some(e) => e,
                None => prompt("Email: ")?,
            };

            let password = match password {
                Some(p) => p,
                None => prompt_password("Password: ")?,
            };

            let mut cfg = load_config().unwrap_or_default();

            if let Some(base) = base_url {
                cfg.auth.base_url = base;
            } else if cfg.auth.base_url.is_empty() {
                // default base URL if nothing set
                cfg.auth.base_url = "http://localhost:3000".to_string();
            }

            let res = api_login(client, &cfg.auth.base_url, &email, &password)
                .await?;
            cfg.auth.token = res.token;

            save_config(&cfg)?;
            // clear session when logging in
            save_session(&Session::default())?;

            println!("Login successful!");
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
                println!("Base URL: {}", cfg.auth.base_url);
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
        OrgCommand::Create { name, slug } => {
            let cfg = ensure_authenticated()?;
            let org = api_create_org(client, &cfg, &name, &slug).await?;

            println!(
                "Organization created: {} (id: {}, slug: {})",
                org.name, org.id, org.slug
            );

            // set as current context
            let mut sess = load_session().unwrap_or_default();
            sess.context.organization_id = Some(org.id);
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
        TeamCommand::Create { name, slug } => {
            let cfg = ensure_authenticated()?;
            let sess = load_session().unwrap_or_default();

            let org_id = sess.context.organization_id.ok_or_else(|| {
                anyhow::anyhow!(
                    "No organization selected. Use `paastel org use` first."
                )
            })?;

            let team =
                api_create_team(client, &cfg, org_id, &name, &slug).await?;

            println!(
                "Team created: {} (id: {}, slug: {})",
                team.name, team.id, team.slug
            );

            let mut sess = sess;
            sess.context.team_id = Some(team.id);
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
                println!("  Base URL    : {}", cfg.auth.base_url);
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

async fn handle_app(cmd: AppCommand, client: &Client) -> Result<()> {
    match cmd {
        AppCommand::Create { name, slug, runtime } => {
            let cfg = ensure_authenticated()?;
            let sess = load_session().unwrap_or_default();

            let org_id = sess.context.organization_id.ok_or_else(|| {
                anyhow::anyhow!(
                    "No organization selected. Use `paastel org use` first."
                )
            })?;
            let team_id = sess.context.team_id.ok_or_else(|| {
                anyhow::anyhow!(
                    "No team selected. Use `paastel team use` first."
                )
            })?;

            let app = api_create_app(
                client,
                &cfg,
                org_id,
                team_id,
                &name,
                &slug,
                runtime.as_deref(),
            )
            .await?;

            println!(
                "Application created: {} (id: {}, slug: {})",
                app.name, app.id, app.slug
            );
        }
    }

    Ok(())
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
            "Base URL is not configured. Set it during login or in config.toml."
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
