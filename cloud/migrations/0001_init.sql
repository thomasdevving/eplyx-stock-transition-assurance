-- Milestone 18 cloud workspace. Users, workspaces and projects are ordinary
-- mutable records. Synced runs, counterexamples and reproduction records are
-- immutable copies of local engine output: triggers below refuse any change
-- to their analytical content, so no code path can rewrite a synced result.

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
    token_sha256 TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE workspace_members (
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('owner', 'member')),
    added_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (workspace_id, user_id)
);

-- Projects are visible to their workspace's members only. There is no public
-- visibility; the single optional demo project is chosen by server config.
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    visibility TEXT NOT NULL DEFAULT 'workspace' CHECK (visibility = 'workspace'),
    created_by TEXT NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Only SHA-256 digests of tokens are stored. `user` tokens come from the CLI
-- device flow; `ci` tokens are scoped to one project and can only sync.
CREATE TABLE api_tokens (
    id TEXT PRIMARY KEY,
    token_sha256 TEXT NOT NULL UNIQUE,
    kind TEXT NOT NULL CHECK (kind IN ('user', 'ci')),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id TEXT REFERENCES projects(id) ON DELETE CASCADE,
    label TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    CHECK ((kind = 'ci') = (project_id IS NOT NULL))
);

CREATE TABLE device_codes (
    device_sha256 TEXT PRIMARY KEY,
    user_code TEXT NOT NULL UNIQUE,
    client TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_poll_at TIMESTAMPTZ,
    approved_by TEXT REFERENCES users(id) ON DELETE CASCADE,
    approved_at TIMESTAMPTZ,
    denied BOOLEAN NOT NULL DEFAULT false,
    consumed_at TIMESTAMPTZ
);

-- Which stable local project IDs feed a cloud project (developer machines
-- and CI checkouts each have their own).
CREATE TABLE project_links (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    local_project_id TEXT NOT NULL,
    linked_by TEXT NOT NULL,
    linked_via TEXT NOT NULL CHECK (linked_via IN ('cli', 'ci')),
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, local_project_id)
);

CREATE TABLE runs (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL,
    local_project_id TEXT NOT NULL,
    core_sha256 TEXT NOT NULL,
    search_sha256 TEXT,
    run_source TEXT,
    run_timestamp TEXT NOT NULL,
    gate_outcome TEXT NOT NULL,
    metadata_text TEXT NOT NULL,
    report_text TEXT NOT NULL,
    bindings_text TEXT NOT NULL,
    manifest_text TEXT NOT NULL,
    config_text TEXT NOT NULL,
    search_text TEXT,
    artifact_sizes TEXT NOT NULL,
    summary TEXT NOT NULL,
    synced_by TEXT NOT NULL,
    synced_via TEXT NOT NULL CHECK (synced_via IN ('cli', 'ci')),
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    search_synced_at TIMESTAMPTZ,
    PRIMARY KEY (project_id, run_id)
);

CREATE TABLE counterexamples (
    project_id TEXT NOT NULL,
    cx_id TEXT NOT NULL,
    run_id TEXT NOT NULL,
    file_sha256 TEXT NOT NULL,
    file_text TEXT NOT NULL,
    summary TEXT NOT NULL,
    synced_by TEXT NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, cx_id),
    FOREIGN KEY (project_id, run_id) REFERENCES runs(project_id, run_id) ON DELETE CASCADE
);

CREATE TABLE reproductions (
    project_id TEXT NOT NULL,
    repro_id TEXT NOT NULL,
    cx_id TEXT NOT NULL,
    file_sha256 TEXT NOT NULL,
    file_text TEXT NOT NULL,
    summary TEXT NOT NULL,
    synced_by TEXT NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (project_id, repro_id),
    FOREIGN KEY (project_id, cx_id) REFERENCES counterexamples(project_id, cx_id) ON DELETE CASCADE
);

CREATE INDEX workspace_members_user ON workspace_members (user_id);
CREATE INDEX projects_workspace ON projects (workspace_id);
CREATE INDEX counterexamples_run ON counterexamples (project_id, run_id);
CREATE INDEX reproductions_cx ON reproductions (project_id, cx_id);

-- A synced run may gain its search result once (absent to present) and have
-- its derived summary refreshed with it. Every other column is fixed.
CREATE FUNCTION eplyx_runs_immutable() RETURNS trigger AS $$
BEGIN
    IF NEW.core_sha256 IS DISTINCT FROM OLD.core_sha256
        OR NEW.local_project_id IS DISTINCT FROM OLD.local_project_id
        OR NEW.metadata_text IS DISTINCT FROM OLD.metadata_text
        OR NEW.report_text IS DISTINCT FROM OLD.report_text
        OR NEW.bindings_text IS DISTINCT FROM OLD.bindings_text
        OR NEW.manifest_text IS DISTINCT FROM OLD.manifest_text
        OR NEW.config_text IS DISTINCT FROM OLD.config_text
        OR NEW.gate_outcome IS DISTINCT FROM OLD.gate_outcome
        OR NEW.run_source IS DISTINCT FROM OLD.run_source
        OR NEW.run_timestamp IS DISTINCT FROM OLD.run_timestamp
        OR NEW.synced_by IS DISTINCT FROM OLD.synced_by
        OR NEW.synced_via IS DISTINCT FROM OLD.synced_via
        OR NEW.synced_at IS DISTINCT FROM OLD.synced_at THEN
        RAISE EXCEPTION 'synced runs are immutable';
    END IF;
    IF OLD.search_sha256 IS NOT NULL AND (
        NEW.search_sha256 IS DISTINCT FROM OLD.search_sha256
        OR NEW.search_text IS DISTINCT FROM OLD.search_text) THEN
        RAISE EXCEPTION 'a synced search result is immutable';
    END IF;
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER runs_immutable BEFORE UPDATE ON runs
    FOR EACH ROW EXECUTE FUNCTION eplyx_runs_immutable();

CREATE FUNCTION eplyx_record_immutable() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'synced records are immutable';
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER counterexamples_immutable BEFORE UPDATE ON counterexamples
    FOR EACH ROW EXECUTE FUNCTION eplyx_record_immutable();
CREATE TRIGGER reproductions_immutable BEFORE UPDATE ON reproductions
    FOR EACH ROW EXECUTE FUNCTION eplyx_record_immutable();
