-- Criação dos tipos ENUM
CREATE TYPE org_role AS ENUM ('owner', 'admin', 'member', 'billing');
CREATE TYPE team_role AS ENUM ('member', 'maintainer', 'lead');

-- Tabela de organizações
CREATE TABLE organizations (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT        NOT NULL,
    slug            TEXT        NOT NULL UNIQUE,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

-- Tabela de usuários globais
CREATE TABLE users (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT        NOT NULL,
    email           TEXT        NOT NULL UNIQUE,
    password_hash   TEXT        NOT NULL,
    is_active       BOOLEAN     NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at   TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ
);

-- Associações usuário ↔ organização
CREATE TABLE organization_memberships (
    organization_id BIGINT     NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id         BIGINT     NOT NULL REFERENCES users(id)         ON DELETE CASCADE,
    role            org_role   NOT NULL DEFAULT 'member',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (organization_id, user_id)
);

-- Tabela de times (squads) dentro da organização
CREATE TABLE teams (
    id              BIGSERIAL PRIMARY KEY,
    organization_id BIGINT     NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    slug            TEXT        NOT NULL,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,

    CONSTRAINT teams_org_slug_unique UNIQUE (organization_id, slug)
);

-- Associações usuário ↔ time
CREATE TABLE team_memberships (
    team_id     BIGINT     NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id     BIGINT     NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role        team_role  NOT NULL DEFAULT 'member',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (team_id, user_id)
);

-- Índices auxiliares

-- Buscar rápido usuários de uma organização
CREATE INDEX idx_org_memberships_user_id
    ON organization_memberships (user_id);

CREATE INDEX idx_org_memberships_org_id
    ON organization_memberships (organization_id);

-- Buscar rápido usuários de um time
CREATE INDEX idx_team_memberships_user_id
    ON team_memberships (user_id);

CREATE INDEX idx_team_memberships_team_id
    ON team_memberships (team_id);




-- Tabela de aplicações do PaaStel
CREATE TABLE apps (
    id               BIGSERIAL PRIMARY KEY,
    organization_id  BIGINT      NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    team_id          BIGINT      REFERENCES teams(id) ON DELETE SET NULL,
    name             TEXT        NOT NULL,
    slug             TEXT        NOT NULL,
    repo_url         TEXT,
    created_by       BIGINT      REFERENCES users(id) ON DELETE SET NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ,

    -- Slug único dentro da organização (várias orgs podem ter "web", mas dentro da mesma org não)
    CONSTRAINT apps_org_slug_unique UNIQUE (organization_id, slug)
);

-- Índices auxiliares

-- Buscar apps por organização
CREATE INDEX idx_apps_organization_id
    ON apps (organization_id);

-- Buscar apps por time (ex: listar apps da squad "devsecops")
CREATE INDEX idx_apps_team_id
    ON apps (team_id);

-- Buscar apps por usuário criador (auditoria / UX)
CREATE INDEX idx_apps_created_by
    ON apps (created_by);

-- Papel do usuário em uma app do PaaStel
CREATE TYPE app_role AS ENUM ('owner', 'maintainer', 'deployer', 'viewer');

-- Vínculo usuário ↔ app
CREATE TABLE app_memberships (
    app_id      BIGINT     NOT NULL REFERENCES apps(id)   ON DELETE CASCADE,
    user_id     BIGINT     NOT NULL REFERENCES users(id)  ON DELETE CASCADE,
    role        app_role   NOT NULL DEFAULT 'viewer',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (app_id, user_id)
);

-- Índices auxiliares para app_memberships
CREATE INDEX idx_app_memberships_user_id
    ON app_memberships (user_id);

CREATE INDEX idx_app_memberships_app_id
    ON app_memberships (app_id);

--------------------------------------------------------------
-- Secrets / env vars das apps
--------------------------------------------------------------

CREATE TABLE app_secrets (
    id               BIGSERIAL PRIMARY KEY,
    app_id           BIGINT      NOT NULL REFERENCES apps(id) ON DELETE CASCADE,

    -- Ambiente lógico: ex: "dev", "staging", "prod"
    environment      TEXT        NOT NULL DEFAULT 'default',

    -- Nome da variável (ex: "DATABASE_URL", "REDIS_PASSWORD")
    key              TEXT        NOT NULL,

    -- Valor da variável.
    -- A aplicação pode criptografar esse valor antes de salvar se quiser.
    value            TEXT        NOT NULL,

    -- Opcional: quem criou/atualizou (bom para auditoria)
    created_by       BIGINT      REFERENCES users(id) ON DELETE SET NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Não permitir duplicidade de chave por app + ambiente
    CONSTRAINT app_secrets_unique_key_per_env UNIQUE (app_id, environment, key)
);

-- Índices auxiliares para app_secrets
CREATE INDEX idx_app_secrets_app_id
    ON app_secrets (app_id);

CREATE INDEX idx_app_secrets_app_env
    ON app_secrets (app_id, environment);

--------------------------------------------------------------
-- Tipos ENUM
--------------------------------------------------------------

-- Status da release (build da app)
CREATE TYPE release_status AS ENUM ('pending', 'built', 'failed');

-- Status do deploy
CREATE TYPE deploy_status AS ENUM (
    'pending',    -- solicitado, aguardando execução
    'running',    -- pipeline em execução
    'succeeded',  -- deploy concluído com sucesso
    'failed',     -- deploy falhou
    'canceled'    -- cancelado pelo usuário/sistema
);

--------------------------------------------------------------
-- Tabela de releases
--------------------------------------------------------------

CREATE TABLE releases (
    id               BIGSERIAL PRIMARY KEY,
    app_id           BIGINT      NOT NULL REFERENCES apps(id) ON DELETE CASCADE,

    -- Versão lógica da release (ex: "v1", "v1.0.3", "2025-11-15.1")
    version          TEXT        NOT NULL,

    -- Dados de origem do código/build
    commit_sha       TEXT,                -- SHA do commit git
    branch           TEXT,                -- branch usada (ex: "main", "develop")
    tag              TEXT,                -- tag do git se houver

    -- Referência do artefato/container gerado
    image_ref        TEXT,                -- ex: "registry.example.com/org/app:hash"

    status           release_status NOT NULL DEFAULT 'pending',
    created_by       BIGINT        REFERENCES users(id) ON DELETE SET NULL,
    created_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW(),

    changelog        TEXT,                -- opcional: notas de release

    -- Garantir unicidade de versão por app
    CONSTRAINT releases_app_version_unique UNIQUE (app_id, version)
);

-- Índices auxiliares em releases
CREATE INDEX idx_releases_app_id
    ON releases (app_id);

CREATE INDEX idx_releases_status
    ON releases (status);

--------------------------------------------------------------
-- Tabela de deploys
--------------------------------------------------------------

CREATE TABLE deploys (
    id               BIGSERIAL PRIMARY KEY,

    app_id           BIGINT      NOT NULL REFERENCES apps(id)     ON DELETE CASCADE,
    release_id       BIGINT      NOT NULL REFERENCES releases(id) ON DELETE RESTRICT,

    -- Ambiente lógico para onde o deploy foi feito
    -- Deve casar com o padrão usado em app_secrets.environment
    environment      TEXT        NOT NULL,

    status           deploy_status NOT NULL DEFAULT 'pending',

    -- Quem iniciou o deploy (via CLI, UI, etc.)
    triggered_by     BIGINT      REFERENCES users(id) ON DELETE SET NULL,

    -- Metadados do alvo (caso queira diferenciar clusters/regiões)
    target_cluster   TEXT,
    target_region    TEXT,

    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at       TIMESTAMPTZ,
    finished_at      TIMESTAMPTZ,

    -- Links auxiliares (ex: URL do job de CI, logs externos, etc.)
    pipeline_url     TEXT,
    logs_url         TEXT,

    -- Mensagem de erro compacta (log completo pode ficar em storage externo)
    error_message    TEXT
);

-- Índices auxiliares em deploys

-- Buscar deploys por app + env (histórico de deploy de uma app em um ambiente)
CREATE INDEX idx_deploys_app_env
    ON deploys (app_id, environment);

-- Buscar deploys por release (ex: "onde essa release foi deployada?")
CREATE INDEX idx_deploys_release_id
    ON deploys (release_id);

-- Buscar deploys recentes por status (para workers de reconciliar/monitorar)
CREATE INDEX idx_deploys_status_created_at
    ON deploys (status, created_at);


--------------------------------------------------------------
-- Tipos ENUM para build
--------------------------------------------------------------

-- Status do build
CREATE TYPE build_status AS ENUM (
    'pending',    -- aguardando execução
    'running',    -- em execução
    'succeeded',  -- concluído com sucesso
    'failed',     -- falhou
    'canceled'    -- cancelado
);

-- Tipo de gatilho do build
CREATE TYPE build_trigger AS ENUM (
    'manual',     -- disparado pela CLI/UI
    'git_push',   -- hook de push
    'api'         -- chamado via API
);

--------------------------------------------------------------
-- Tabela de builds (build_jobs)
--------------------------------------------------------------

CREATE TABLE build_jobs (
    id               BIGSERIAL PRIMARY KEY,

    app_id           BIGINT      NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    release_id       BIGINT      REFERENCES releases(id) ON DELETE SET NULL,

    status           build_status NOT NULL DEFAULT 'pending',
    trigger          build_trigger NOT NULL DEFAULT 'manual',

    triggered_by     BIGINT      REFERENCES users(id) ON DELETE SET NULL,

    -- contexto do código/fontes
    commit_sha       TEXT,
    branch           TEXT,
    tag              TEXT,

    -- referência do artefato gerado (se aplicável)
    image_ref        TEXT,

    -- onde esse build rodou (runner)
    runner_name      TEXT,   -- ex: "runner-01"
    runner_type      TEXT,   -- ex: "k8s", "local", "docker"

    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at       TIMESTAMPTZ,
    finished_at      TIMESTAMPTZ,

    -- links externos
    logs_url         TEXT,   -- ex: URL para logs completos em S3/Minio
    pipeline_url     TEXT,   -- ex: URL do job em um runner externo

    -- erro resumido
    error_message    TEXT
);

-- Índices auxiliares para build_jobs
CREATE INDEX idx_build_jobs_app_id
    ON build_jobs (app_id);

CREATE INDEX idx_build_jobs_status_created_at
    ON build_jobs (status, created_at);

CREATE INDEX idx_build_jobs_release_id
    ON build_jobs (release_id);

--------------------------------------------------------------
-- Tabela de steps do build
--------------------------------------------------------------

-- Status do step (pode reaproveitar build_status, mas deixo separado
-- se quiser diferenciar depois; por simplicidade, vamos reutilizar build_status):

CREATE TABLE build_steps (
    id               BIGSERIAL PRIMARY KEY,

    build_id         BIGINT      NOT NULL REFERENCES build_jobs(id) ON DELETE CASCADE,

    -- ordem de execução do step (1, 2, 3, ...)
    position         INTEGER     NOT NULL,

    name             TEXT        NOT NULL,  -- ex: "fetch source", "build image", "run tests"

    status           build_status NOT NULL DEFAULT 'pending',

    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at       TIMESTAMPTZ,
    finished_at      TIMESTAMPTZ,

    logs_url         TEXT,     -- logs específicos do step (S3/Minio/etc.)
    error_message    TEXT,

    CONSTRAINT build_steps_position_positive CHECK (position > 0)
);

-- Índices auxiliares para build_steps
CREATE INDEX idx_build_steps_build_id
    ON build_steps (build_id);

CREATE INDEX idx_build_steps_build_id_position
    ON build_steps (build_id, position);


--------------------------------------------------------------
-- (Opcional) Tabela de logs de build em chunks
-- Se você decidir não guardar logs no Postgres, pode pular essa parte.
--------------------------------------------------------------

CREATE TABLE build_logs (
    id            BIGSERIAL PRIMARY KEY,

    build_id      BIGINT      NOT NULL REFERENCES build_jobs(id)  ON DELETE CASCADE,
    step_id       BIGINT      REFERENCES build_steps(id)          ON DELETE CASCADE,

    -- índice do chunk (0, 1, 2, ...)
    chunk_index   INTEGER     NOT NULL,

    -- conteúdo do log (texto puro)
    content       TEXT        NOT NULL,

    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT build_logs_chunk_unique UNIQUE (build_id, step_id, chunk_index)
);

CREATE INDEX idx_build_logs_build_id
    ON build_logs (build_id);

CREATE INDEX idx_build_logs_step_id
    ON build_logs (step_id);

