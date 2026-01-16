CREATE DATABASE connect_management;

\c connect_management
CREATE SEQUENCE IF NOT EXISTS groups_id_seq;

CREATE TABLE groups
(
    group_id    integer                  default nextval('groups_id_seq'::regclass) not null
        primary key,
    name        varchar(100)                                                        not null
        unique,
    description text,
    created_at  timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at  timestamp with time zone default CURRENT_TIMESTAMP
);

create table servers
(
    id            serial
        primary key,
    name          varchar(255),
    group_id      integer
        constraint fk_group
            references groups
            on update cascade on delete set null,
    ssh_user      varchar(100)             default 'root'::character varying not null,
    ip            varchar(45)                                                not null,
    port          integer                  default 22                        not null,
    password_hash text                                                       not null,
    created_at    timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at    timestamp with time zone default CURRENT_TIMESTAMP,
    constraint unique_ip_port
        unique (ip, port)
);

-- 创建序列
CREATE SEQUENCE IF NOT EXISTS cron_jobs_id_seq;

-- 创建 cronjobs 主表
CREATE TABLE IF NOT EXISTS cronjobs
(
    id              integer                  DEFAULT nextval('cron_jobs_id_seq'::regclass) NOT NULL
        CONSTRAINT cronjobs_pkey PRIMARY KEY,
    name            varchar(255),
    cron_expression varchar(100)                                                           NOT NULL,
    server_id       integer
        CONSTRAINT fk_server
            REFERENCES servers(id)
            ON UPDATE CASCADE ON DELETE CASCADE,
    group_id        integer
        CONSTRAINT fk_group
            REFERENCES groups(group_id)
            ON UPDATE CASCADE ON DELETE CASCADE,
    command         text                                                                   NOT NULL,
    enabled         boolean                  DEFAULT true                                  NOT NULL,
    timeout         integer                  DEFAULT 300,
    retry_count     integer                  DEFAULT 0,
    description     text,
    last_executed_at timestamp with time zone,
    next_execute_at timestamp with time zone,
    created_at      timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at      timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT check_server_or_group
        CHECK ((server_id IS NOT NULL AND group_id IS NULL) OR
               (server_id IS NULL AND group_id IS NOT NULL) OR
               (server_id IS NOT NULL AND group_id IS NOT NULL))
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_cronjobs_server_id ON cronjobs(server_id);
CREATE INDEX IF NOT EXISTS idx_cronjobs_group_id ON cronjobs(group_id);
CREATE INDEX IF NOT EXISTS idx_cronjobs_enabled ON cronjobs(enabled);
CREATE INDEX IF NOT EXISTS idx_cronjobs_next_execute ON cronjobs(next_execute_at) 
    WHERE enabled = true;

-- ============================================
-- 触发器函数：通知任务变更（包含 DELETE）
-- ============================================
-- CREATE OR REPLACE FUNCTION notify_cronjob_changes()
-- RETURNS TRIGGER AS $$
-- BEGIN
--     IF (TG_OP = 'INSERT') THEN
--         PERFORM pg_notify('cronjob_changes', json_build_object(
--             'operation', 'INSERT',
--             'id', NEW.id,
--             'enabled', NEW.enabled,
--             'next_execute_at', NEW.next_execute_at
--         )::text);
--         RETURN NEW;
--
--     ELSIF (TG_OP = 'UPDATE') THEN
--         -- 关键字段变化时才通知
--         IF (OLD.enabled != NEW.enabled OR
--             OLD.cron_expression != NEW.cron_expression OR
--             OLD.next_execute_at IS DISTINCT FROM NEW.next_execute_at OR
--             OLD.command != NEW.command) THEN
--             PERFORM pg_notify('cronjob_changes', json_build_object(
--                 'operation', 'UPDATE',
--                 'id', NEW.id,
--                 'enabled', NEW.enabled,
--                 'next_execute_at', NEW.next_execute_at
--             )::text);
--         END IF;
--         RETURN NEW;
--
--     ELSIF (TG_OP = 'DELETE') THEN
--         -- DELETE 操作通知
--         PERFORM pg_notify('cronjob_changes', json_build_object(
--             'operation', 'DELETE',
--             'id', OLD.id
--         )::text);
--         RETURN OLD;
--     END IF;
-- END;
-- $$ LANGUAGE plpgsql;
--
-- -- 绑定触发器（包含 DELETE）
-- -- DROP TRIGGER IF EXISTS cronjob_changes_trigger ON cronjobs;
-- CREATE TRIGGER cronjob_changes_trigger
-- AFTER INSERT OR UPDATE OR DELETE ON cronjobs
-- FOR EACH ROW
-- EXECUTE FUNCTION notify_cronjob_changes();

-- ============================================
-- 触发器函数：自动更新 updated_at
-- ============================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- DROP TRIGGER IF EXISTS update_cronjobs_updated_at ON cronjobs;
CREATE TRIGGER update_cronjobs_updated_at
BEFORE UPDATE ON cronjobs
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- 执行日志表（可选，用于记录任务执行历史）
-- ============================================
CREATE TABLE IF NOT EXISTS cronjob_logs
(
    log_id      bigserial                                      NOT NULL
        CONSTRAINT cronjob_logs_pkey PRIMARY KEY,
    job_id      integer                                        NOT NULL
        CONSTRAINT fk_cronjob
            REFERENCES cronjobs(id)
            ON UPDATE CASCADE ON DELETE CASCADE,
    executed_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    status      varchar(20)                                    NOT NULL, -- 'success', 'failed', 'timeout'
    output      text,
    error       text,
    duration_ms integer
);

CREATE INDEX IF NOT EXISTS idx_cronjob_logs_job_id ON cronjob_logs(job_id);
CREATE INDEX IF NOT EXISTS idx_cronjob_logs_executed_at ON cronjob_logs(executed_at);