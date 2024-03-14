use anyhow::Context;

#[derive(Debug, thiserror::Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

pub trait TodoRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: i32,
    pub text: String,
    pub completed: bool,
}

#[derive(
    Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, validator::Validate,
)]
pub struct CreateTodo {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    text: String,
}

#[cfg(test)]
impl CreateTodo {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

#[derive(
    Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, validator::Validate,
)]
pub struct UpdateTodo {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    text: Option<String>,
    completed: Option<bool>,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

type TodoDatas = std::collections::HashMap<i32, Todo>;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: std::sync::Arc<std::sync::RwLock<TodoDatas>>,
}

impl TodoRepositoryForMemory {
    pub fn new() -> Self {
        Self {
            store: std::sync::Arc::default(),
        }
    }

    fn write_store_ref(&self) -> std::sync::RwLockWriteGuard<TodoDatas> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> std::sync::RwLockReadGuard<TodoDatas> {
        self.store.read().unwrap()
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let todo = Todo::new(id, payload.text.clone());
        store.insert(id, todo.clone());
        todo
    }

    fn find(&self, id: i32) -> Option<Todo> {
        let store = self.read_store_ref();
        store.get(&id).map(|todo| todo.clone())
    }

    fn all(&self) -> Vec<Todo> {
        let store = self.read_store_ref();
        Vec::from_iter(store.values().map(|todo| todo.clone()))
    }

    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref();
        let todo = store.get(&id).context(RepositoryError::NotFound(id))?;
        let text = payload.text.unwrap_or(todo.text.clone());
        let completed = payload.completed.unwrap_or(todo.completed);
        let todo = Todo {
            id,
            text,
            completed,
        };
        store.insert(id, todo.clone());
        Ok(todo)
    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn todo_crud_scerario() {
        let text = "todo text".to_string();
        let id = 1;
        let expected = Todo::new(id, text.clone());

        let repository = TodoRepositoryForMemory::new();
        let todo = repository.create(CreateTodo { text });
        assert_eq!(expected, todo);

        let todo = repository.find(todo.id).unwrap();
        assert_eq!(expected, todo);

        let todo = repository.all();
        assert_eq!(vec![expected], todo);

        let text = "update todo text".to_string();
        let todo = repository
            .update(
                1,
                UpdateTodo {
                    text: Some(text.clone()),
                    completed: Some(false),
                },
            )
            .expect("failed update todo");

        assert_eq!(
            Todo {
                id,
                text,
                completed: true,
            },
            todo
        );

        let res = repository.delete(id);
        assert!(res.is_ok());
    }
}
