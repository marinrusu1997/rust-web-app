use crate::ctx::Ctx;
use crate::model::Result;
use crate::model::base::DbBmc;
use crate::model::{ModelManager, base};
use lib_auth::pwd::{ContentToHash, hash_pwd};
use modql::field::{Fields, HasSeaFields};
use sea_query::{Expr, Iden, Query, SimpleExpr};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::postgres::PgRow;
use uuid::Uuid;

#[derive(Clone, Debug, FromRow, Fields, Serialize)]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Deserialize)]
pub struct UserForCreate {
    pub username: String,
    pub password: String,
}

#[derive(FromRow, Fields)]
pub struct UserForLogin {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub password_salt: Uuid,
    pub token_salt: Uuid,
}

#[derive(FromRow, Fields)]
pub struct UserForAuth {
    pub id: i64,
    pub username: String,
    pub token_salt: Uuid,
}

pub trait UserBy: for<'r> FromRow<'r, PgRow> + Unpin + Send + HasSeaFields {} // market trait
impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

#[derive(Iden)]
enum UserIden {
    Id,
    Username,
    Password,
}

pub struct UserBmc;

impl DbBmc for UserBmc {
    const TABLE_NAME: &'static str = "user";
}

impl UserBmc {
    pub async fn get<E: UserBy>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
    where
        E: UserBy,
    {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn first_by_username<E>(
        _ctx: &Ctx,
        mm: &ModelManager,
        username: &str,
    ) -> Result<Option<E>>
    where
        E: UserBy,
    {
        let db = mm.db();

        // -- Build query
        let mut query = Query::select();
        query
            .from(Self::table_ref())
            .columns(E::sea_idens())
            .and_where(Expr::col(UserIden::Username).eq(username));

        // -- Exec query
        let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
        let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
            .fetch_optional(db)
            .await?;

        Ok(entity)
    }

    pub async fn update_password(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        password_clear: &str,
    ) -> Result<()> {
        let db = mm.db();

        let user: UserForLogin = Self::get(ctx, mm, id).await?;
        let password = hash_pwd(&ContentToHash {
            content: password_clear.to_string(),
            salt: Uuid::from(user.password_salt),
        })?;

        let mut query = Query::update();
        query
            .table(Self::table_ref())
            .value(UserIden::Password, SimpleExpr::from(password))
            .and_where(Expr::col(UserIden::Id).eq(id));

        let (sql, values) = query.build_sqlx(sea_query::PostgresQueryBuilder);
        sqlx::query_with(&sql, values)
            .execute(db)
            .await?
            .rows_affected();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::_dev_utils;
    use crate::ctx::Ctx;
    use crate::model::user::{User, UserBmc};
    use anyhow::{Context, Result};
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_first_ok_demo1() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_username = "demo1";

        let user: User = UserBmc::first_by_username(&ctx, &mm, fx_username)
            .await?
            .context(format!("user '{fx_username}' not found"))?;

        assert_eq!(user.username, fx_username);

        Ok(())
    }
}
