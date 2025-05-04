#![allow(unused)] // For early development.

// region:    --- Modules

mod ctx;
mod error;
mod log;
mod model;
mod web;

pub use self::error::Result;

use crate::model::ModelManager;
use crate::web::mw_auth::mw_ctx_resolve;
use crate::web::mw_res_map::mw_reponse_map;
use crate::web::{routes_login, routes_static};
use axum::{middleware, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize ModelManager.
    let mm = ModelManager::new().await?;

    // -- Define Routes
    // let routes_rpc = rpc::routes(mm.clone())
    //   .route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_login::routes())
        // .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_reponse_map))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    // region:    --- Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr)
        .await
        .expect(&format!("Failed to bind on {}", addr));
    println!("Listening on {}", addr);

    axum::serve(listener, routes_all).await.unwrap();
    // endregion: --- Start Server

    Ok(())
}
