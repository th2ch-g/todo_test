use super::*;

pub async fn create_label<T: crate::repositories::label::LabelRepository>(
    ValidatedJson(payload): ValidatedJson<CreateLabel>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let label = repository
        .create(payload.name)
        .await
        .or(Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok((axum::http::StatusCode::CREATED, axum::Json(label)))
}

pub async fn all_label<T: crate::repositories::label::LabelRepository>(
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let labels = repository.all().await.unwrap();
    Ok((axum::http::StatusCode::OK, axum::Json(labels)))
}

pub async fn delete_label<T: crate::repositories::label::LabelRepository>(
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> axum::http::StatusCode {
    repository
        .delete(id)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .unwrap_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, validator::Validate)]
pub struct CreateLabel {
    #[validate(length(min=1, message="cannot be empty"))]
    #[validate(length(max=100, message="over text length"))]
    name: String,
}
