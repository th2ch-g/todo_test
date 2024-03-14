mod handlers;
mod repositories;

#[tokio::main]
async fn main() {
    let log_level = std::env::var("RUST_LOG").unwrap_or("info".to_string());
    std::env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let database_url = &std::env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
    tracing::debug!("start connect database..");
    let pool = sqlx::PgPool::connect(database_url)
        .await
        .expect("fail connect database");

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
