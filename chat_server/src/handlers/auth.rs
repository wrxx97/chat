use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{
    models::{CreateUser, SignInUser},
    AppError, AppState,
};

#[derive(Debug, Serialize, Deserialize)]
struct AuthOutput {
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SignInUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify(input).await?;

    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => Err(AppError::NotFound("user not found".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use axum::http::StatusCode;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateUser::new("acme", "Tian Chen", "tyr@acme.org", "123456");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signin_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "test@qq.com";
        let fullname = "wrxx";
        let password = "wrxx";

        let input = CreateUser::new("default", fullname, email, password);
        let ret = signup_handler(State(state.clone()), Json(input))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::CREATED);

        let input = SignInUser {
            email: email.to_string(),
            password: password.to_string(),
        };

        let ret = signin_handler(State(state.clone()), Json(input))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::OK);

        Ok(())
    }
}
