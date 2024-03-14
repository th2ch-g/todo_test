mod handlers;
mod repositories;

#[tokio::main]
async fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let repository = crate::repositories::TodoRepositoryForMemory::new();
    let app = create_app(repository);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: crate::repositories::TodoRepository>(repository: T) -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(root))
        .route(
            "/todos",
            axum::routing::post(crate::handlers::create_todo::<T>)
                .get(crate::handlers::all_todo::<T>),
        )
        .route(
            "/todos/:id",
            axum::routing::get(crate::handlers::find_todo::<T>)
                .delete(crate::handlers::delete_todo::<T>)
                .patch(crate::handlers::update_todo::<T>),
        )
        .layer(axum::extract::Extension(std::sync::Arc::new(repository)))
}

async fn root() -> &'static str {
    "Hello world"
}

#[cfg(test)]
mod test {
    use super::*;
    use tower::ServiceExt;

    fn build_todo_req_with_json(
        path: &str,
        method: axum::http::Method,
        json_body: String,
    ) -> axum::http::Request<axum::body::Body> {
        axum::http::Request::builder()
            .uri(path)
            .method(method)
            .header(
                axum::http::header::CONTENT_TYPE,
                mime::APPLICATION_JSON.as_ref(),
            )
            .body(axum::body::Body::from(json_body))
            .unwrap()
    }

    fn build_todo_req_with_empty(
        method: axum::http::Method,
        path: &str,
    ) -> axum::http::Request<axum::body::Body> {
        axum::http::Request::builder()
            .uri(path)
            .method(method)
            .body(axum::body::Body::empty())
            .unwrap()
    }

    async fn res_to_todo(res: axum::http::Response) -> crate::repositories::Todo {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let todo: crate::repositories::Todo =
            serder_json::from_str(&body).expect("cannot format todo instance");
        todo
    }

    #[tokio::test]
    async fn should_created_todo() {
        let expected = crate::repositories::Todo::new(1, "should_return_created_todo".to_string());
        let repositories = TodoRepositoryForMemory::new();
        let req = build_todo_req_with_json(
            "/todos",
            axum::http::Method::POST,
            r#"{ "text": "should_return_created_todo" }"#.to_string(),
        );
        let res = create_app(repository).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_find_todo() {
        let expected = crate::repositories::Todo::new(1, "should_find_todo".to_string());
        let repository = TodoRepositoryForMemory::new();
        repository.create(crate::repositories::CreateTodo::new(
            "should_find_todo".to_string(),
        ));
        let req = build_todo_req_with_empty(axum::http::Method::GET, "/todos/1");
        let res = create_app(repository).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_get_all_todos() {
        let expected = crate::repositories::Todo::new(1, "should_get_all_todos".to_string());
        let repository = TodoRepositoryForMemory::new();
        repository.create(crate::repositories::CreateTodo::new(
            "should_get_all_todos".to_string(),
        ));
        let req = build_todo_req_with_empty(axum::http::Method::GET, "/todos");
        let res = create_app(repository).oneshot(req).await.unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let todo: Vec<crate::repositories::Todo> =
            serde_json::from_str(&body).expect("cannot convert todo");
        assert_eq!(vec![expected], todo);
    }
}
