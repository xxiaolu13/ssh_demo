
use sqlx::PgPool;
use tracing::log::{error,info};
use crate::domain::server::{CreateGroupServiceTerminal, CreateSingleServiceTerminal, ServiceTerminal};
use crate::utils::crypto::passwd_encryption;
use crate::repository::servergroup::get_group_by_id_db;




pub async fn get_all_servers_db(p0: &PgPool) ->  Result<Vec<ServiceTerminal>, anyhow::Error>{
    let rows = sqlx::query_as!(
        ServiceTerminal,
        "select id,name,group_id,ssh_user,ip,port,password_hash as password from servers"
    ).fetch_all(p0).await?;
    match rows.len(){
        0 => Err(anyhow::Error::msg("get all servers not found")),
        _ => Ok(rows)
    }
}

pub async fn get_server_by_id_db(p0: &PgPool, id: i32) -> Result<ServiceTerminal, anyhow::Error>{
    let row = sqlx::query_as!(
        ServiceTerminal,
        "select id,name,group_id,ssh_user,ip,port,password_hash as password from servers where id=$1",
        id
    ).fetch_one(p0).await?;
    Ok(row)
}

pub async fn get_server_by_group_id_db(p0: &PgPool, id: i32) -> Result<Vec<ServiceTerminal>, anyhow::Error>{
    let row = sqlx::query_as!(
        ServiceTerminal,
        "select id,name,group_id,ssh_user,ip,port,password_hash as password from servers where group_id=$1",
        id
    ).fetch_all(p0).await?;
    Ok(row)
}


pub async fn create_single_server_db(p0: &PgPool, server: CreateSingleServiceTerminal) -> Result<CreateSingleServiceTerminal, anyhow::Error> {
    let ssh_user = if let Some(e) = server.ssh_user{
        e
    }else{
        "root".to_string()
    };
    let port = if let Some(e) = server.port{
        e
    }else{
        22
    };
    if let Some(e) = server.group_id{
        let _ = get_group_by_id_db(p0,e).await.map_err(|e| {
            error!("Create server but Failed to fetch group by id: {:?}", e);
            anyhow::anyhow!("Group with id {} not found", e)
        })?;
    }
    let password = passwd_encryption(server.password.clone())?;
    let row = sqlx::query_as!(
        CreateSingleServiceTerminal,
        r#"
        INSERT INTO servers (name,group_id,ssh_user,ip,port,password_hash)
        VALUES ($1, $2,$3,$4,$5,$6)
        RETURNING name,group_id,ssh_user,ip,port,password_hash as password
        "#,
        server.name.clone(),
        server.group_id,
        ssh_user,
        server.ip.clone(),
        port,
        password
    ).fetch_one(p0).await?;
    Ok(row)
}


pub async fn create_group_server_db(p0: &PgPool,server: CreateGroupServiceTerminal) -> Result<Vec<CreateSingleServiceTerminal>, anyhow::Error> {
    // 密码加密，用户端口设置默认值
    let password = passwd_encryption(server.password.clone())?;
    let ssh_user = if let Some(e) = server.ssh_user{
        e
    }else{
        "root".to_string()
    };
    let port = if let Some(e) = server.port{
        e
    }else{
        22
    };

    let mut count = 1;
    let server_ip_list = server.ip;

    let mut rows = Vec::new();
    for this_ip in server_ip_list{
        let name = server.name.as_ref().map(|prefix| format!("{}{}", prefix, count));
        let row = sqlx::query_as!(
        CreateSingleServiceTerminal,
        r#"
        INSERT INTO servers (name,group_id,ssh_user,ip,port,password_hash)
        VALUES ($1, $2,$3,$4,$5,$6)
        RETURNING name,group_id,ssh_user,ip,port,password_hash as password
        "#,
        name,
        server.group_id,
        ssh_user.clone(),
        this_ip.clone(),
        port,
        password.clone()
    ).fetch_one(p0).await
            .map_err(|e| {
            error!("Failed to insert server with ip {}: {}", this_ip, e);
            e
        });
        match row {
            Ok(r) => {
                rows.push(r);
                info!("Successfully inserted server with ip: {}", this_ip);
            },
            Err(_) => {
                count += 1;
                continue;
            }
        }
        count += 1;
    }
    Ok(rows)
}


pub async fn delete_single_server_by_id_db(p0: &PgPool, id: i32) -> Result<String, anyhow::Error>{
    let row = sqlx::query!("delete from servers where id=$1",id).execute(p0).await?;
    Ok(format!("Successfully deleted {:?} group", row))
}