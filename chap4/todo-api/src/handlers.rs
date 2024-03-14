pub async fn create_todo<T: crate::repositories::TodoRepository>(
    ValidatedJson(payload): ValidatedJson<crate::repositories::CreateTodo>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository
        .create(payload)
        .await
        .or(Err(axum::http::StatusCode::NOT_FOUND))?;
    Ok((axum::http::StatusCode::CREATED, axum::Json(todo)))
}

pub async fn find_todo<T: crate::repositories::TodoRepository>(
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository
        .find(id)
        .await
        .or(Err(axum::http::StatusCode::NOT_FOUND))?;
    Ok((axum::http::StatusCode::OK, axum::Json(todo)))
}

pub async fn all_todo<T: crate::repositories::TodoRepository>(
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository.all().await.unwrap();
    Ok((axum::http::StatusCode::OK, axum::Json(todo)))
}

pub async fn update_todo<T: crate::repositories::TodoRepository>(
    axum::extract::Path(id): axum::extract::Path<i32>,
    ValidatedJson(payload): ValidatedJson<crate::repositories::UpdateTodo>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> Result<impl axum::response::IntoResponse, axum::http::StatusCode> {
    let todo = repository
        .update(id, payload)
        .await
        .or(Err(axum::http::StatusCode::NOT_FOUND))?;
    Ok((axum::http::StatusCode::CREATED, axum::Json(todo)))
}

pub async fn delete_todo<T: crate::repositories::TodoRepository>(
    axum::extract::Path(id): axum::extract::Path<i32>,
    axum::extract::Extension(repository): axum::extract::Extension<std::sync::Arc<T>>,
) -> axum::http::StatusCode {
    repository
        .delete(id)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .unwrap_or(axum::http::StatusCode::NOT_FOUND)
}

#[derive(Debug)]
pub struct ValidatedJson<T>(T);

#[axum::async_trait]
impl<T, B> axum::extract::FromRequest<B> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + validator::Validate,
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<axum::BoxError>,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request(
        req: &mut axum::extract::RequestParts<B>,
    ) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req)
            .await
            .map_err(|rejection| {
                let message = format!("Json parse error: {}", rejection);
                (axum::http::StatusCode::BAD_REQUEST, message)
            })?;
        value.validate().map_err(|rejection| {
            let message = format!("validation error: {}", rejection).replace("\n", ", ");
            (axum::http::StatusCode::BAD_REQUEST, message)
        })?;
        Ok(ValidatedJson(value))
    }
}
