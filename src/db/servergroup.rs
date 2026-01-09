use sqlx::PgPool;

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
    Ok(format!("Successfully deleted {:?} group", row))
}


pub async fn update_group_by_id_db(p0: &PgPool,id: i32,newgroup: UpdateGroup) -> Result<Group, anyhow::Error>{
    let base_group = get_group_by_id_db(p0,id).await?;
    // 处理 name：如果提供了新值则使用，否则保持原值
    let name = newgroup.name.unwrap_or(base_group.name);

    // 这个地方肯定有bug，如果传入None那么这个None是否是需要更新的值，如果原来是Some，则我不想修改，默认会传入None，那么就把原来的值改了
    let description = newgroup.description.or(base_group.description);

    let row = sqlx::query_as!(
        Group,
        "update groups set name = $1,description= $2 where group_id = $3 returning group_id,name,description",
        name,description,id
    ).fetch_one(p0).await?;

    Ok(row)
}