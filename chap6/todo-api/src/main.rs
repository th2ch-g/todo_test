mod handlers;
mod repositories;

#[tokio::main]
async fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let database_url = &std::env::var("DATABASE_URL").expect("undefined DATABASE_URL");
    tracing::debug!("start connect database");
    let pool = sqlx::PgPool::connect(database_url)
        .await
        .expect("fail connect database");
    let app = create_app(
        crate::repositories::todo::TodoRepositoryForDb::new(pool.clone()),
        crate::repositories::label::LabelRepositoryForDb::new(pool.clone())
    );
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<Todo: crate::repositories::todo::TodoRepository,
   Label: crate::repositories::label::LabelRepository>
(todo_repository: Todo, label_repository: Label) -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(root))
        .route("/todos", axum::routing::post(crate::handlers::todo::create_todo::<Todo>)
               .get(crate::handlers::todo::all_todo::<Todo>))
        .route("/todos/:id", axum::routing::get(crate::handlers::todo::delete_todo::<Todo>)
               .delete(crate::handlers::todo::delete_todo::<Todo>)
               .patch(crate::handlers::todo::update_todo::<Todo>)
        )
        .route("/labels", axum::routing::post(crate::handlers::label::create_label::<Label>)
               .get(crate::handlers::label::all_label::<Label>)
        )
        .route("/labels/:id", axum::routing::delete(crate::handlers::label::delete_label::<Label>))
        .layer(axum::extract::Extension(std::sync::Arc::new(todo_repository)))
        .layer(axum::extract::Extension(std::sync::Arc::new(label_repository)))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Origin::exact("http://localhost:3001".parse().unwrap()))
                .allow_methods(tower_http::cors::Any)
                .allow_headers(vec![hyper::header::CONTENT_TYPE])
        )
}

async fn root() -> &'static str {
    "hello world"
}
