use super::error::Result;
use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::{base, ModelManager};
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsBool, OpValsInt64, OpValsString};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Fields, Serialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

#[derive(Deserialize, Fields)]
pub struct TaskForCreate {
    pub title: String,
}

#[derive(Deserialize, Default, Fields)]
pub struct TaskForUpdate {
    pub title: Option<String>,
    pub done: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct TaskFilter {
    id: Option<OpValsInt64>,

    title: Option<OpValsString>,
    done: Option<OpValsBool>,
}

pub struct TaskBmc;

impl DbBmc for TaskBmc {
    const TABLE_NAME: &'static str = "task";
}

impl TaskBmc {
    pub async fn create(ctx: &Ctx, mm: &ModelManager, task_c: TaskForCreate) -> Result<i64> {
        base::create::<Self, _>(ctx, mm, task_c).await
    }

    pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        base::get::<Self, _>(ctx, mm, id).await
    }

    pub async fn list(
        ctx: &Ctx,
        mm: &ModelManager,
        filters: Option<Vec<TaskFilter>>,
        list_options: Option<ListOptions>,
    ) -> Result<Vec<Task>> {
        base::list::<Self, _, _>(ctx, mm, filters, list_options).await
    }

    pub async fn update(
        ctx: &Ctx,
        mm: &ModelManager,
        id: i64,
        task_u: TaskForUpdate,
    ) -> Result<()> {
        base::update::<Self, _>(ctx, mm, id, task_u).await
    }

    pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
        base::delete::<Self>(ctx, mm, id).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::error::Error;
    use crate::_dev_utils;
    use crate::ctx::Ctx;
    use crate::model::task::{Task, TaskBmc, TaskFilter, TaskForCreate, TaskForUpdate};
    use anyhow::Result;
    use modql::filter::ListOptions;
    use serde_json::json;
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
    async fn test_list_all_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &[
            "test_list_all_ok-task title 1",
            "test_list_all_ok-task title 2",
        ];
        let tasks = _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        let tasks_list: Vec<Task> = TaskBmc::list(&ctx, &mm, None, None)
            .await?
            .into_iter()
            .filter(|t| t.title.starts_with("test_list_all_ok-task"))
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
    async fn test_list_by_filter_ok() -> Result<()> {
        let mm = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_titles = &[
            "test_list_by_filter_ok-task title 01.a",
            "test_list_by_filter_ok-task title 01.b",
            "test_list_by_filter_ok-task title 02.a",
            "test_list_by_filter_ok-task title 02.b",
            "test_list_by_filter_ok-task title 03",
        ];
        let tasks = _dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

        let filters: Vec<TaskFilter> = serde_json::from_value(json!([
            {
                "title": {
                    "$endsWith": ".a",
                    "$containsAny": ["01", "02"]
                }
            },
            {
                "title": {
                    "$contains": "03"
                }
            }
        ]))?;
        let list_options: ListOptions = serde_json::from_value(json!({
            "order_bys": "!id"
        }))?;
        let found_tasks_list: Vec<Task> =
            TaskBmc::list(&ctx, &mm, Some(filters), Some(list_options)).await?;

        assert_eq!(found_tasks_list.len(), 3);
        assert!(found_tasks_list[0].title.ends_with("03"));
        assert!(found_tasks_list[1].title.ends_with("02.a"));
        assert!(found_tasks_list[2].title.ends_with("01.a"));

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
        let task_c = TaskForUpdate {
            title: Some(fx_title.to_string()),
            done: None,
        };
        TaskBmc::update(&ctx, &mm, task.id, task_c).await?;

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
                Err(Error::EntityNotFound {
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
        let task_u = TaskForUpdate {
            title: Some("test_update_not_found title".to_string()),
            done: None,
        };

        let result = TaskBmc::update(&ctx, &mm, task_id, task_u).await;
        assert!(
            matches!(
                result,
                Err(Error::EntityNotFound {
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
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100
                })
            ),
            "EntityNotFound error not returned as expected"
        );

        Ok(())
    }
}
