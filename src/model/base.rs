use crate::ctx::Ctx;
use crate::model::Result;
use crate::model::{Error, ModelManager};
use sqlx::postgres::PgRow;
use sqlx::FromRow;

pub trait DbBmc {
    const TABLE_NAME: &'static str;
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();

    let sql = format!("SELECT * FROM {} WHERE id = $1", MC::TABLE_NAME);
    let entity: E = sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or(Error::EntityNotFound {
            entity: MC::TABLE_NAME,
            id,
        })?;

    Ok(entity)
}

pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where
    MC: DbBmc,
    E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
{
    let db = mm.db();

    let sql = format!("SELECT * FROM {} ORDER BY id", MC::TABLE_NAME);
    let entities: Vec<E> = sqlx::query_as(&sql).fetch_all(db).await?;

    Ok(entities)
}

pub async fn delete<MC: DbBmc>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
    let db = mm.db();

    let sql = format!("DELETE FROM {} WHERE id = $1", MC::TABLE_NAME);
    let count = sqlx::query(&sql)
        .bind(id)
        .execute(db)
        .await?
        .rows_affected();

    if count == 0 {
        return Err(Error::EntityNotFound {
            entity: MC::TABLE_NAME,
            id,
        });
    }

    Ok(())
}
