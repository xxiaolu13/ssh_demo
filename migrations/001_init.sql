CREATE DATABASE server_management;

\c server_management
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

CREATE SEQUENCE IF NOT EXISTS cron_jobs_id_seq;

CREATE TABLE cron_jobs
(
    id              integer                  default nextval('cron_jobs_id_seq'::regclass) not null
        primary key,
    name            varchar(255),
    cron_expression varchar(100)                                                           not null,
    server_id       integer
        constraint fk_server
            references servers(id)
            on update cascade on delete cascade,
    group_id        integer
        constraint fk_group
            references groups(group_id)
            on update cascade on delete cascade,
    command         text                                                                   not null,
    created_at      timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at      timestamp with time zone default CURRENT_TIMESTAMP,
    constraint check_server_or_group
        check ((server_id is not null and group_id is null) or
               (server_id is null and group_id is not null) or
               (server_id is not null and group_id is not null))
);

-- 为外键列创建索引，提升查询性能
CREATE INDEX idx_cron_jobs_server_id ON cron_jobs(server_id);
CREATE INDEX idx_cron_jobs_group_id ON cron_jobs(group_id);