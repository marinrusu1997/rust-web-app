use super::error::{Error, Result};
use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::{base, ModelManager};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
}

#[derive(Deserialize)]
pub struct TaskForCreate {
    pub title: String,
}

pub struct TaskBmc;

impl DbBmc for TaskBmc {
    const TABLE_NAME: &'static str = "task";
}

impl TaskBmc {
    pub async fn create(_ctx: &Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        let db = mm.db();
        let (id,) = sqlx::query_as::<_, (i64,)>(&format!(
            "INSERT INTO {} (title) VALUES ($1) RETURNING id",
            Self::TABLE_NAME
        ))
        .bind(task_c.title)
        .fetch_one(db)
        .await?;

        Ok(id)
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list(ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
        base::list::<Self, _>(ctx, mm).await
    }

    pub async fn update(_ctx: &Ctx, mm: &ModelManager, task: Task) -> Result<()> {
        let db = mm.db();
        let count = sqlx::query(&format!(
            "UPDATE {} SET title = $1 WHERE id = $2",
            Self::TABLE_NAME
        ))
        .bind(task.title)
        .bind(task.id)
        .execute(db)
        .await?
        .rows_affected();

        if count == 0 {
            return Err(Error::EntityNotFound {
                entity: Self::TABLE_NAME,
                id: task.id,
            });
        }

        Ok(())
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::_dev_utils;
    use crate::ctx::Ctx;
    use crate::model::task::{Task, TaskBmc, TaskForCreate};
    use anyhow::Result;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok title";

        let task_c = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBmc::create(&ctx, &mm, task_c).await?;

        let task = TaskBmc::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        TaskBmc::delete(&ctx, &mm, id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_list_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &["test_list_ok-task title 1", "test_list_ok-task title 2"];
        let tasks = _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        let tasks_list: Vec<Task> = TaskBmc::list(&ctx, &mm)
            .await?
            .into_iter()
            .filter(|t| t.title.starts_with("test_list_ok-task"))
            .collect();
        assert_eq!(
            tasks_list.len(),
            fx_titles.len(),
            "Task list length mismatch"
        );

        for task in tasks.iter() {
            TaskBmc::delete(&ctx, &mm, task.id).await?;
        }

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_update_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let mut fx_title = "test_create_ok title";

        let task_c = TaskForCreate {
            title: fx_title.to_string(),
        };
        let id = TaskBmc::create(&ctx, &mm, task_c).await?;

        let mut task = TaskBmc::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        fx_title = "test_update_ok title";
        task.title = fx_title.to_string();
        TaskBmc::update(&ctx, &mm, task).await?;

        let task = TaskBmc::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        TaskBmc::delete(&ctx, &mm, id).await?;

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_get_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        let res = TaskBmc::get(&ctx, &mm, fx_id).await;
        assert!(
            matches!(
                res,
                Err(super::Error::EntityNotFound {
                    entity: "task",
                    id: 100
                })
            ),
            "EntityNotFound error not returned as expected"
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_update_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();

        let task_id = 100;
        let task = Task {
            id: task_id,
            title: "test_update_not_found title".to_string(),
        };

        let result = TaskBmc::update(&ctx, &mm, task).await;
        assert!(
            matches!(
                result,
                Err(super::Error::EntityNotFound {
                    entity: "task",
                    id: task_id
                })
            ),
            "EntityNotFound error not returned as expected"
        );

        Ok(())
    }

    #[serial]
    #[tokio::test]
    async fn test_delete_err_not_found() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_id = 100;

        let res = TaskBmc::delete(&ctx, &mm, fx_id).await;
        assert!(
            matches!(
                res,
                Err(super::Error::EntityNotFound {
                    entity: "task",
                    id: 100
                })
            ),
            "EntityNotFound error not returned as expected"
        );

        Ok(())
    }
}
