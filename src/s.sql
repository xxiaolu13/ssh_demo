create table groups
(
    group_id    integer                  default nextval('groups_id_seq'::regclass) not null
        primary key,
    name        varchar(100)                                                        not null
        unique,
    description text,
    created_at  timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at  timestamp with time zone default CURRENT_TIMESTAMP
)

create table servers
(
    id            serial
        primary key,
    name          varchar(255),
    group_id      integer
        constraint fk_group
            references groups
            on update cascade on delete set null,
    "user"        varchar(100)             default 'root'::character varying,
    ip            varchar(45)                         not null,
    port          integer                  default 22 not null,
    password_hash text                                not null,
    created_at    timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at    timestamp with time zone default CURRENT_TIMESTAMP,
    constraint unique_ip_port
        unique (ip, port)
);