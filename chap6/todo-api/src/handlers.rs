pub mod label;
pub mod todo;

#[derive(Debug)]
pub struct ValidatedJson<T>(T);

#[axum::async_trait]
impl <T, B> axum::extract::FromRequest<B> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + validator::Validate,
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<axum::BoxError>,
{
    type Rejection = (axum::http::StatusCode, String);

    async fn from_request(req: &mut axum::extract::RequestParts<B>) -> Result<Self, Self::Rejection> {
        let axum::Json(value) = axum::Json::<T>::from_request(req).await.map_err(|rejection| {
            (axum::http::StatusCode::BAD_REQUEST, format!("Json parse error: {}", rejection))
        })?;
        value.validate().map_err(|rejection| {
            (axum::http::StatusCode::BAD_REQUEST, format!("validation error: {}", rejection).replace("\n", ", "))
        });
        Ok(ValidatedJson(value))
    }
}
