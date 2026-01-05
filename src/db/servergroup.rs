use sqlx::{query, PgPool};
use sqlx::postgres::PgQueryResult;
use crate::model::servergroup::*;

pub async fn get_all_groups_db(p0: &PgPool) -> Result<Vec<Group>,anyhow::Error>{
    let rows = sqlx::query_as!(
        Group,"select group_id,name,description from groups").fetch_all(p0).await?;
    match rows.len(){
        0 => Err(anyhow::Error::msg("get all servers not found")),
        _ => Ok(rows)
    }
}

pub async fn get_group_by_id_db(
    pool: &PgPool,
    id: i32
) -> Result<Group, anyhow::Error> {
    let row = sqlx::query_as!(
        Group,
        r#"
        SELECT group_id,name,description FROM groups WHERE group_id = $1
        "#,
        id
    ).fetch_one(pool).await?;
    Ok(row)
}


pub async fn create_group_db(
    pool: &PgPool,
    group: CreateGroup
) -> Result<Group, anyhow::Error> {
    let row = sqlx::query_as!(
        Group,
        r#"
        INSERT INTO groups (name, description)
        VALUES ($1, $2)
        RETURNING group_id, name, description
        "#,
        group.name,
        group.description
    )
        .fetch_one(pool)
        .await?;
    Ok(row)
}


pub async fn delete_group_by_id_db(p0: &PgPool,id: i32) -> Result<String, anyhow::Error>{
    let row = sqlx::query!("delete from groups where group_id=$1",id).execute(p0).await?;
    Ok(format!("Successfully deleted {:?} row", row))
}