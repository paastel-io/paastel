//! paastel-build - build de imagem Docker usando bollard
//! Empacota TODO o contexto respeitando .dockerignore.

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use bollard::Docker;
use bollard::auth::DockerCredentials;
use bollard::image::PushImageOptions;
use bollard::models::PushImageInfo;
use bollard::query_parameters::BuildImageOptions;
use bytes::Bytes;
use clap::Parser;
use futures_util::stream::StreamExt;
use globset::{Glob, GlobMatcher};
use http_body_util::{Either, Full};
use walkdir::WalkDir;

/// CLI para buildar uma imagem Docker usando bollard,
/// empacotando TODO o contexto e respeitando .dockerignore.
///
/// Exemplo:
///   paastel-build \
///     --context . \
///     --dockerfile Dockerfile \
///     --image localhost:5000/teste/nginx:dev \
///     --pull
#[derive(Parser, Debug)]
#[command(name = "paastel-build")]
#[command(about = "Build Docker image using bollard (with .dockerignore)", long_about = None)]
struct Cli {
    /// Diretório raiz do build context (onde está o Dockerfile e .dockerignore).
    #[arg(long, default_value = ".")]
    context: String,

    /// Nome do Dockerfile dentro do contexto (ex: Dockerfile, docker/Dockerfile).
    #[arg(long, default_value = "Dockerfile")]
    dockerfile: String,

    /// Nome completo da imagem (ex: localhost:5000/org/team/app:tag).
    #[arg(long)]
    image: String,

    /// Sempre tentar dar pull da base (equivalente a --pull no docker build).
    #[arg(long)]
    pull: bool,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("paastel-build error: {err:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Cli::parse();

    let context_dir = Path::new(&args.context);
    if !context_dir.exists() {
        return Err(anyhow!(
            "Context directory '{}' não existe",
            context_dir.display()
        ));
    }

    let dockerfile_path = context_dir.join(&args.dockerfile);
    if !dockerfile_path.exists() {
        return Err(anyhow!(
            "Dockerfile '{}' não encontrado em '{}'",
            args.dockerfile,
            context_dir.display()
        ));
    }

    println!("==> Conectando ao Docker daemon (socket defaults)...");
    let docker = Docker::connect_with_socket_defaults()
        .context("Falha ao conectar ao Docker daemon (socket)")?;

    println!("==> Preparando build context (tar+gzip em memória)...");
    let compressed = build_context_tar_gz(context_dir)?;

    println!("==> Iniciando build da imagem: {}", args.image);
    println!("    Context   : {}", context_dir.display());
    println!("    Dockerfile: {}", args.dockerfile);
    println!("    pull base : {}", args.pull);
    println!();

    // Usa a API nova: BuildImageOptionsBuilder em vez da struct deprecated.
    // let builder = BuildImageOptionsBuilder::default();
    let options = BuildImageOptions {
        dockerfile: args.dockerfile.clone(),
        t: Some(args.image.clone()), // <-- AQUI é onde o tag é setado
        rm: true,
        pull: if args.pull { Some("true".to_string()) } else { None },
        ..Default::default()
    };

    // builder.clone()
    //     .t(args.image.as_str())
    //     .dockerfile(args.dockerfile.as_str())
    //     .rm(true)
    //     .pull(if args.pull { "true" } else { "false" });

    // let options: BuildImageOptions = builder.build();

    // Corpo do tar.gz em memória.
    let body = Either::Left(Full::new(Bytes::from(compressed)));

    let mut stream = docker.build_image(options, None, Some(body));

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(output) => {
                if let Some(stream) = output.stream {
                    print!("{stream}");
                }
                if let Some(error) = output.error {
                    eprintln!("Docker build error: {error}");
                }
            }
            Err(e) => {
                return Err(anyhow!("Erro durante o stream do build: {e}"));
            }
        }
    }

    println!();
    println!("✅ Build finalizado para imagem: {}", args.image);

    // Push para o registry
    push_image_to_registry(&docker, &args.image).await?;

    Ok(())
}

/// Faz o push da imagem para o registry.
///
/// `image_full` é algo como:
/// - "sample-nginx:dev"
/// - "localhost:5000/teste/nginx:dev"
async fn push_image_to_registry(
    docker: &Docker,
    image_full: &str,
) -> Result<()> {
    let (repo, tag) = split_image(image_full);

    println!();
    println!("==> Realizando push da imagem: {}", image_full);
    println!("    Repo: {}", repo);
    println!("    Tag : {}", tag);

    let options = Some(PushImageOptions::<String> {
        tag: tag.clone(),
        ..Default::default()
    });

    // Sem credenciais (útil para registry local / público)
    let creds: Option<DockerCredentials> = None;

    let mut stream = docker.push_image(&repo, options, creds);

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(status) => match status {
                PushImageInfo { error: Some(err), .. } => {
                    eprintln!("❌ Docker push error: {}", err);
                }
                PushImageInfo {
                    status: Some(msg),
                    progress: Some(prog),
                    ..
                } => {
                    println!("→ {} | {}", msg, prog);
                }
                PushImageInfo {
                    status: Some(msg), progress: None, ..
                } => {
                    println!("→ {}", msg);
                }
                _ => {}
            },
            Err(e) => {
                return Err(anyhow!("Erro durante o push da imagem: {e}"));
            }
        }
    }

    println!("✅ Push finalizado para {}", image_full);
    Ok(())
}

/// Divide "repo:tag" em (repo, tag).
///
/// Exemplos:
/// - "sample-nginx:dev" → ("sample-nginx", "dev")
/// - "localhost:5000/teste/nginx:dev" → ("localhost:5000/teste/nginx", "dev")
/// - "nginx" → ("nginx", "latest")
fn split_image(image: &str) -> (String, String) {
    // Regra: último ':' depois do último '/' separa a tag
    let last_colon = image.rfind(':');
    let last_slash = image.rfind('/');

    match (last_colon, last_slash) {
        (Some(c), Some(s)) if c > s => {
            let repo = &image[..c];
            let tag = &image[c + 1..];
            (repo.to_string(), tag.to_string())
        }
        (Some(c), None) => {
            let repo = &image[..c];
            let tag = &image[c + 1..];
            (repo.to_string(), tag.to_string())
        }
        _ => (image.to_string(), "latest".to_string()),
    }
}

/// Representa as regras do .dockerignore, com suporte a:
/// - ordem das regras (última que casa vence)
/// - padrões normais (excluir)
/// - padrões começando com '!' (reinclude)
struct Dockerignore {
    rules: Vec<(GlobMatcher, bool)>, // bool = is_exclude (true) ou include (!pattern => false)
}

impl Dockerignore {
    fn is_ignored(&self, rel_path: &str) -> bool {
        let mut matched_any = false;
        let mut result_is_exclude = false;

        for (matcher, is_exclude) in &self.rules {
            if matcher.is_match(rel_path) {
                matched_any = true;
                result_is_exclude = *is_exclude;
            }
        }

        matched_any && result_is_exclude
    }
}

/// Carrega .dockerignore (se existir) e monta as regras.
fn load_dockerignore(context_dir: &Path) -> Result<Option<Dockerignore>> {
    let path = context_dir.join(".dockerignore");
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(&path).with_context(|| {
        format!("Falha ao ler .dockerignore em {}", path.display())
    })?;

    let mut rules = Vec::new();

    for raw_line in contents.lines() {
        let mut line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut is_exclude = true;

        if let Some(stripped) = line.strip_prefix('!') {
            line = stripped.trim_start();
            is_exclude = false;
        }

        let glob = Glob::new(line).with_context(|| {
            format!("Padrão inválido em .dockerignore: {}", raw_line)
        })?;
        let matcher = glob.compile_matcher();

        rules.push((matcher, is_exclude));
    }

    if rules.is_empty() { Ok(None) } else { Ok(Some(Dockerignore { rules })) }
}

/// Cria um tar.gz em memória contendo TODO o contexto,
/// respeitando .dockerignore.
fn build_context_tar_gz(context_dir: &Path) -> Result<Vec<u8>> {
    let dockerignore = load_dockerignore(context_dir)?;

    let mut tar_builder = tar::Builder::new(Vec::new());

    for entry in WalkDir::new(context_dir).follow_links(false).into_iter() {
        let entry = entry.with_context(|| "Erro ao caminhar build context")?;
        let path = entry.path();

        if path == context_dir {
            continue;
        }

        let rel = path.strip_prefix(context_dir).with_context(|| {
            format!("Falha ao calcular path relativo para {}", path.display())
        })?;

        let rel_str = rel.to_string_lossy().replace('\\', "/");

        if let Some(di) = &dockerignore {
            if di.is_ignored(&rel_str) {
                continue;
            }
        }

        if entry.file_type().is_file() {
            let full_path = path;
            let rel_path = Path::new(&rel_str);

            tar_builder
                .append_path_with_name(full_path, rel_path)
                .with_context(|| {
                    format!(
                        "Falha ao adicionar {} ao tar",
                        full_path.display()
                    )
                })?;
        }
    }

    let uncompressed = tar_builder
        .into_inner()
        .context("Falha ao finalizar tar (into_inner)")?;

    let mut encoder = flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    );
    encoder
        .write_all(&uncompressed)
        .context("Falha ao escrever dados no GzEncoder")?;
    let compressed =
        encoder.finish().context("Falha ao finalizar compressão gzip")?;

    Ok(compressed)
}
