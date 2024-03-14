use super::*;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForDb {
    pool: sqlx::PgPool,
}

impl TodoRepositoryForDb {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[axum::async_trait]
impl TodoRepository for TodoRepositoryForDb {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<TodoEntity> {
        let tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TodoFromRow>(
            r#"
insert into todos(text, completed)
values ($1, false)
returning *
            "#
        )
        .bind(payload.text.clone())
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"
insert into todo_labels(todo_id, label_id)
select $1, id
from unnest($2) as t(id);
            "#
        )
        .bind(row.id)
        .bind(payload.labels)
        .execute(&self.pool)
        .await?;

        tx.commit().await?;

        let todo = self.find(row.id).await?;
        Ok(todo)
    }

    async fn find(&self, id: i32) -> anyhow::Result<TodoEntity> {
        let items = sqlx::query_as::<_, TodoWithLabelFromRow>(
            r#"
select todos.*, labels.id as label_id, labels.name as label_name
from todos
    left outer join todo_labels tl on todos.id = tl.todo_id
    left outer join labels on labels.id = tl.label_id
where todos.id=$1
            "#
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        let todos = fold_entities(items);
        let todo = todos.first().ok_or(RepositoryError::NotFound(id))?;
        Ok(todo.clone())
    }

    async fn all(&self) -> anyhow::Result<Vec<TodoEntity>> {
        let items = sqlx::query_as::<_, TodoWithLabelFromRow>(
            r#"
select todos.*, labels.id as label_id, labels.name as label_name
from todos
    left outer join todo_labels tl on todos.id = tl.todo_id
    left outer join labels on labels.id = tl.label_id
order by todos.id desc;
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(fold_entities(items))
    }

    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<TodoEntity> {
        let tx = self.pool.begin().await?;

        let old_todo = self.find(id).await?;
        sqlx::query(
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

        if let Some(labels) = payload.labels {
            sqlx::query(
                r#"
delete from todo_labels where todo_id=$1
                "#
            )
            .bind(id)
            .execute(&self.pool)
            .await?;

            sqlx::query(
                r#"
insert into todo_labels (todo_id, label_id)
select $1, id
from unnest($2) as t(id)
                "#
            )
            .bind(id)
            .bind(labels)
            .execute(&self.pool)
            .await?;
        };

        tx.commit().await?;
        let todo = self.find(id).await?;
        Ok(todo)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        let tx = self.pool.begin().await?;
        sqlx::query(
            r#"
delete from todo_labels where todo_id=$1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

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

        tx.commit().await?;

        Ok(())
    }
}


#[axum::async_trait]
pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<TodoEntity>;
    async fn find(&self, id: i32) -> anyhow::Result<TodoEntity>;
    async fn all(&self) -> anyhow::Result<Vec<TodoEntity>>;
    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<TodoEntity>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
struct TodoFromRow {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
struct TodoWithLabelFromRow {
    id: i32,
    text: String,
    completed: bool,
    label_id: Option<i32>,
    label_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct TodoEntity {
    pub id: i32,
    pub text: String,
    pub completed: bool,
    pub labels: Vec<crate::repositories::label::Label>,
}

fn fold_entities(rows: Vec<TodoWithLabelFromRow>) -> Vec<TodoEntity> {
    let mut rows = rows.iter();
    let mut accum: Vec<TodoEntity> = vec![];
    'outer: while let Some(row) = rows.next() {
        let mut todos = accum.iter_mut();
        while let Some(todo) = todos.next() {
            if todo.id == row.id {
                todo.labels.push(crate::repositories::label::Label {
                    id: row.label_id.unwrap(),
                    name: row.label_name.clone().unwrap(),
                });
                continue 'outer;
            }
        }

        let labels = if row.label_id.is_some() {
            vec![crate::repositories::label::Label {
                id: row.label_id.unwrap(),
                name: row.label_name.clone().unwrap(),
            }]
        } else {
            vec![]
        };

        accum.push(TodoEntity {
            id: row.id,
            text: row.text.clone(),
            completed: row.completed,
            labels,
        });
    }
    accum
}


#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, validator::Validate)]
pub struct CreateTodo {
    #[validate(length(min=1, message="cannot be empty"))]
    #[validate(length(max=100, message="over text length"))]
    text: String,
    labels: Vec<i32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, validator::Validate)]
pub struct UpdateTodo {
    #[validate(length(min=1, message="cannot be empty"))]
    #[validate(length(max=100, message="over text length"))]
    text: Option<String>,
    completed: Option<bool>,
    labels: Option<Vec<i32>>,
}


