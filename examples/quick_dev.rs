#![allow(unused)] // For beginning only.

use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:3000")?;

    // hc.do_get("/index.html").await?.print().await?;

    println!("--- Login");
    let req_login = hc.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "password": "demo1"
        }),
    );
    req_login.await?.print().await?;

    println!("--- Create Task");
    let req_create_task = hc.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "create_task",
            "params": {
                "data": {
                    "title": "task AAA"
                }
            }
        }),
    );
    req_create_task.await?.print().await?;

    println!("--- Update Task");
    let req_update_task = hc.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "update_task",
            "params": {
                "id": 1000, // Hardcode the task id.
                "data": {
                    "title": "task BB"
                }
            }
        }),
    );
    req_update_task.await?.print().await?;

    println!("--- Delete Task");
    let req_delete_task = hc.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "delete_task",
            "params": {
                "id": 1000 // Harcode the task id
            }
        }),
    );
    req_delete_task.await?.print().await?;

    println!("--- List Tasks");
    let req_list_tasks = hc.do_post(
        "/api/rpc",
        json!({
            "id": 1,
            "method": "list_tasks",
            "params": {}
        }),
    );
    req_list_tasks.await?.print().await?;

    println!("--- Logoff");
    let req_logoff = hc.do_post(
        "/api/logoff",
        json!({
            "logoff": true
        }),
    );
    req_logoff.await?.print().await?;

    Ok(())
}
