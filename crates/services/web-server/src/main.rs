mod config;
mod error;
mod log;
mod web;

pub use self::error::{Error, Result};
use config::web_config;

use crate::web::mw_auth::{mw_ctx_require, mw_ctx_resolve};
use crate::web::mw_res_map::mw_reponse_map;
use crate::web::routes_rpc::RpcState;
use crate::web::{routes_login, routes_rpc, routes_static};
use axum::{Router, middleware};
use lib_core::_dev_utils;
use lib_core::model::ModelManager;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;
// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // -- FOR DEV ONLY
    _dev_utils::init_dev().await;

    // Initialize ModelManager.
    let mm = ModelManager::new().await?;

    // -- Define Routes
    let routes_rpc = routes_rpc::routes(RpcState { mm: mm.clone() })
        .route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_login::routes(mm.clone()))
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_reponse_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    // region:    --- Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .expect(&format!("Failed to bind on {}", addr));
    info!("Listening on {}", addr);

    axum::serve(listener, routes_all).await.unwrap();
    // endregion: --- Start Server

    Ok(())
}
