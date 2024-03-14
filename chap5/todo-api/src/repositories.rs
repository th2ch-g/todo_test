#[derive(Debug, thiserror::Error)]
enum RepositoryError {
    #[error("Unexpected Error: {0}")]
    Unexpected(String),
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

#[derive(Debug, Clone)]
pub struct TodoRepositoryForDb {
    pool: sqlx::PgPool,
}

impl TodoRepositoryForDb {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[axum::async_trait]
impl TodoRepository for TodoRepositoryForDb {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as::<_, Todo>(
            r#"
insert into todos (text, completed)
values ($1, false)
returning *
            "#
            )
            .bind(payload.text.clone())
            .fetch_one(&self.pool)
            .await?;

        Ok(todo)
    }

    async fn find(&self, id: i32) -> anyhow::Result<Todo> {
        let todo = sqlx::query_as::<_, Todo> (
            r#"
select * from todos where id=$1
            "#
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(todo)
    }

    async fn all(&self) -> anyhow::Result<Vec<Todo>> {
        let todos = sqlx::query_as::<_, Todo>(
            r#"
select * from todos
order by id desc;
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(todos)
    }

    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let old_todo = self.find(id).await?;
        let todo = sqlx::query_as::<_, Todo>(
            r#"
update todos set text=$1, completed=$2
where id=$3
returning *
            "#
        )
        .bind(payload.text.unwrap_or(old_todo.text))
        .bind(payload.completed.unwrap_or(old_todo.completed))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(todo)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
delete from todos where id=$1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}

#[axum::async_trait]
pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<Todo>;
    async fn find(&self, id: i32) -> anyhow::Result<Todo>;
    async fn all(&self) -> anyhow::Result<Vec<Todo>>;
    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub completed: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, validator::Validate)]
pub struct CreateTodo {
    #[validate(length(min=1, message="can not be empty"))]
    #[validate(length(max=100, message="Over text length"))]
    text: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, validator::Validate)]
pub struct UpdateTodo {
    #[validate(length(min=1, message="cannot be empty"))]
    #[validate(length(max=100, message="over text length"))]
    text: Option<String>,
    completed: Option<bool>,
}
