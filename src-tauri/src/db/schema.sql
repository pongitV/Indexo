-- Schema do perfil local do OrganizadorApp.
-- Fica em data/profile.db, ao lado do executavel.

CREATE TABLE IF NOT EXISTS scan_sessions (
    id            TEXT PRIMARY KEY,
    root_path     TEXT NOT NULL,
    started_at    TEXT NOT NULL,
    finished_at   TEXT,
    files_scanned INTEGER NOT NULL DEFAULT 0,
    status        TEXT NOT NULL DEFAULT 'running' -- running | done | cancelled | error
);

CREATE TABLE IF NOT EXISTS files (
    id                  TEXT PRIMARY KEY,
    session_id          TEXT NOT NULL REFERENCES scan_sessions(id),
    original_path       TEXT NOT NULL,
    filename            TEXT NOT NULL,
    extension_declared  TEXT,
    extension_detected  TEXT,          -- lido pelos bytes via `infer`, nao confia no nome
    size_bytes          INTEGER NOT NULL,
    content_hash        TEXT,          -- sha256, calculado sob demanda (nao no scan inicial)
    created_at          TEXT,
    modified_at         TEXT,
    last_scanned_at     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_files_session ON files(session_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_files_path ON files(original_path);

CREATE TABLE IF NOT EXISTS categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    color       TEXT,
    parent_id   TEXT REFERENCES categories(id),
    created_by  TEXT NOT NULL DEFAULT 'auto', -- auto | user
    created_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS file_categories (
    file_id       TEXT NOT NULL REFERENCES files(id),
    category_id   TEXT NOT NULL REFERENCES categories(id),
    confidence    REAL NOT NULL DEFAULT 0.0,     -- 0.0 a 1.0
    assigned_by   TEXT NOT NULL,                 -- heuristic | embedding | llm | user
    assigned_at   TEXT NOT NULL,
    PRIMARY KEY (file_id, category_id)
);

-- Regras aprendidas: o coracao da precisao crescente do app.
-- Toda correcao do usuario reforca (ou cria) uma linha aqui.
CREATE TABLE IF NOT EXISTS classification_rules (
    id                TEXT PRIMARY KEY,
    pattern_type      TEXT NOT NULL,   -- extension | filename_regex | content_keyword | embedding_cluster
    pattern_value     TEXT NOT NULL,
    category_id       TEXT NOT NULL REFERENCES categories(id),
    confidence_weight REAL NOT NULL DEFAULT 0.5,
    created_from      TEXT NOT NULL,   -- learned | user
    hit_count         INTEGER NOT NULL DEFAULT 0,
    created_at        TEXT NOT NULL,
    updated_at        TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_rules_pattern_type ON classification_rules(pattern_type);

CREATE TABLE IF NOT EXISTS user_corrections (
    id               TEXT PRIMARY KEY,
    file_id          TEXT NOT NULL REFERENCES files(id),
    old_category_id  TEXT REFERENCES categories(id),
    new_category_id  TEXT NOT NULL REFERENCES categories(id),
    corrected_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS move_log (
    id          TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL REFERENCES scan_sessions(id),
    file_id     TEXT NOT NULL REFERENCES files(id),
    from_path   TEXT NOT NULL,
    to_path     TEXT NOT NULL,
    moved_at    TEXT NOT NULL,
    undone      INTEGER NOT NULL DEFAULT 0 -- 0/1, gravado ANTES do move de fato acontecer
);
CREATE INDEX IF NOT EXISTS idx_move_log_session ON move_log(session_id);

CREATE TABLE IF NOT EXISTS embeddings_cache (
    content_hash  TEXT NOT NULL,
    model_version TEXT NOT NULL,
    vector        BLOB NOT NULL,
    PRIMARY KEY (content_hash, model_version)
);

CREATE TABLE IF NOT EXISTS settings (
    key    TEXT PRIMARY KEY,
    value  TEXT NOT NULL
);
