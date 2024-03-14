use super::*;
pub async fn create_todo<T: crate::repositories::todo::TodoRepository>(
    ValidatedJson(payload): ValidatedJson<crate::repositories::todo::CreateTodo>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository
        .create(payload)
        .await
        .or(Err(axum::http::StatusCode::NOT_FOUND))?;

    Ok((axum::http::StatusCode::CREATED, axum::Json(todo)))
}

pub async fn find_todo<T: crate::repositories::todo::TodoRepository>(
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository.find(id).await.or(Err(axum::http::StatusCode::NOT_FOUND))?;
    Ok((axum::http::StatusCode::OK, axum::Json(todo)))
}

pub async fn all_todo<T: crate::repositories::todo::TodoRepository>(
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository.all().await.unwrap();
    Ok((axum::http::StatusCode::OK, axum::Json(todo)))
}

pub async fn update_todo<T: crate::repositories::todo::TodoRepository>(
    axum::extract::Path(id): axum::extract::Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::repositories::todo::UpdateTodo>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository
        .update(id, payload)
        .await
        .or(Err(axum::http::StatusCode::NOT_FOUND))?;
    Ok((axum::http::StatusCode::CREATED, axum::Json(todo)))
}

pub async fn delete_todo<T: crate::repositories::todo::TodoRepository>(
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> axum::http::StatusCode {
    repository
        .delete(id)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .unwrap_or(axum::http::StatusCode::NOT_FOUND)
}

