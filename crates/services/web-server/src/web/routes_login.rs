use crate::web::{self, Error, Result, remove_token_cookie};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use lib_auth::pwd::{self, ContentToHash, SchemeStatus};
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use lib_core::model::user::{UserBmc, UserForLogin};
use serde::Deserialize;
use serde_json::{Value, json};
use tower_cookies::Cookies;
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login_handler))
        .route("/api/logoff", post(api_logoff_handler))
        .with_state(mm)
}

// region:    --- Login
async fn api_login_handler(
    State(mm): State<ModelManager>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_login_handler", "HANDLER");

    let LoginPayload {
        username,
        pwd: pwd_clear,
    } = payload;
    let root_ctx = Ctx::root_ctx();

    // -- Get the user.
    let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;
    let user_id = user.id;

    let scheme_status = pwd::validate_pwd(
        ContentToHash {
            salt: user.password_salt,
            content: pwd_clear.clone(),
        },
        &user.password,
    )
    .await
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

    // -- Update password scheme if needed.
    if let SchemeStatus::Outdated = scheme_status {
        debug!("Upgrading password scheme");
        UserBmc::update_password(&root_ctx, &mm, user_id, &pwd_clear).await?;
    }

    // -- Set web token.
    web::set_token_cookie(&cookies, &user.username, user.token_salt)?;

    // Create the success body.
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}
// endregion: --- Login

// region:    --- Logoff
async fn api_logoff_handler(
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_logoff_handler", "HANDLER");
    let should_logoff = payload.logoff;

    if should_logoff {
        remove_token_cookie(&cookies)?;
    }

    // Create the success body.
    let body = Json(json!({
        "result": {
            "logged_off": should_logoff
        }
    }));

    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LogoffPayload {
    logoff: bool,
}
// endregion: --- Logoff
