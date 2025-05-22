use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::web::rpc::task_rpc::{create_task, delete_task, list_tasks, update_task};
use crate::web::{Error, Result};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use tracing::debug;

mod task_rpc;

#[derive(Deserialize, Debug)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    id: i64,
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsIded {
    id: i64,
}

#[derive(Debug, Clone)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

async fn rpc_handler(
    ctx: Ctx,
    State(mm): State<ModelManager>,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    let rpc_info = RpcInfo {
        id: rpc_req.id.clone(),
        method: rpc_req.method.clone(),
    };

    let mut response = rpc_handler_inner(ctx, mm, rpc_req).await.into_response();
    response.extensions_mut().insert(rpc_info);

    response
}

macro_rules! exec_rpc_fn {
    // With params
    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
        let rpc_fn_name = stringify!($rpc_fn);
        let params = $rpc_params.ok_or(Error::RpcMissingParams {
            method: rpc_fn_name.to_string(),
        })?;
        let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
            method: rpc_fn_name.to_string(),
        })?;

        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};

    // Without params
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };
}

async fn rpc_handler_inner(ctx: Ctx, mm: ModelManager, rpc_req: RpcRequest) -> Result<Json<Value>> {
    debug!("--> rpc_req: {:?}", rpc_req);

    let result_json: Value = match rpc_req.method.as_str() {
        "create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_req.params),
        "list_tasks" => exec_rpc_fn!(list_tasks, ctx, mm),
        "update_task" => exec_rpc_fn!(update_task, ctx, mm, rpc_req.params),
        "delete_task" => exec_rpc_fn!(delete_task, ctx, mm, rpc_req.params),

        _ => return Err(Error::RpcMethodUnknown(rpc_req.method)),
    };

    let body_response = json!({
        "id": rpc_req.id,
        "result": result_json,
    });

    Ok(Json(body_response))
}
