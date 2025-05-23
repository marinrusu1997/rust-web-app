use crate::web::{AUTH_TOKEN, set_token_cookie};
use crate::web::{Error, Result};
use axum::extract::{FromRequestParts, OptionalFromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use lib_auth::token::{Token, validate_web_token};
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use lib_core::model::user::{UserBmc, UserForAuth};
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

pub async fn mw_ctx_require(ctx: Result<CtxW>, req: Request, next: Next) -> Result<Response> {
    debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

pub async fn mw_ctx_resolve(
    mm: State<ModelManager>,
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let result_ctx = ctx_resolve(mm, &cookies).await;
    if result_ctx.is_err() && !matches!(result_ctx, Err(CtxExtError::TokenNotInCookie)) {
        cookies.remove(Cookie::from(AUTH_TOKEN));
    }

    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

async fn ctx_resolve(mm: State<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;

    let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

    let user: UserForAuth = UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
        .await
        .map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
        .ok_or(CtxExtError::UserNotFound)?;

    validate_web_token(&token, user.token_salt).map_err(|_| CtxExtError::FailValidate)?;

    set_token_cookie(cookies, &user.username, user.token_salt)
        .map_err(|_| CtxExtError::CannotSetTokenCookie)?;

    Ctx::new(user.id)
        .map(CtxW)
        .map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

impl<S: Send + Sync> FromRequestParts<S> for CtxW {
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        debug!("{:<12} - Ctx Required", "EXTRACTOR");

        parts
            .extensions
            .get::<CtxExtResult>()
            .ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .clone()
            .map_err(Error::CtxExt)
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for CtxW {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>> {
        debug!("{:<12} - Ctx Optional", "EXTRACTOR");

        let ctx_ext_result = parts.extensions.get::<CtxExtResult>();

        if let Some(ctx_ext_result) = ctx_ext_result {
            if let Ok(ctx_ext_result) = ctx_ext_result {
                return Ok(Some(ctx_ext_result.clone()));
            }
        }

        Ok(None)
    }
}

// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
