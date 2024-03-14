mod handlers;
mod repositories;

#[tokio::main]
async fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let database_url = &std::env::var("DATABASE_URL").expect("undefined url");
    tracing::debug!("start connect database...");
    let pool = sqlx::PgPool::connect(database_url)
        .await
        .expect("failed connect database");
    let repository = crate::repositories::TodoRepositoryForDb::new(pool.clone());
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
        .route("/todos", axum::routing::post(crate::handlers::create_todo::<T>)
               .get(crate::handlers::all_todo::<T>))
        .route("/todos/:id",
               axum::routing::get(crate::handlers::find_todo::<T>)
               .delete(crate::handlers::delete_todo::<T>)
               .patch(crate::handlers::update_todo::<T>),
        )
        .layer(axum::extract::Extension(std::sync::Arc::new(repository)))
        .layer(tower_http::cors::CorsLayer::new()
               .allow_origin(tower_http::cors::Origin::exact("http://localhost:3001".parse().unwrap()))
               .allow_methods(tower_http::cors::Any)
               .allow_headers(vec![hyper::header::CONTENT_TYPE])
        )
}

async fn root() -> &'static str {
    "hello world"
}


