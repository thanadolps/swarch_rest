use poem::{
    error::InternalServerError, listener::TcpListener, middleware, web::Data, EndpointExt, Result,
    Route, Server,
};
use poem_openapi::{param::Path, payload::Json, ApiResponse, Object, OpenApi, OpenApiService};
use sqlx::{FromRow, SqlitePool};

#[derive(Object, Debug, FromRow)]
struct Task {
    #[oai(read_only)]
    id: i64,
    title: String,
    description: Option<String>,
    #[oai(default)]
    completed: bool,
}

#[derive(ApiResponse)]
enum TaskResponse {
    #[oai(status = 200)]
    Ok(Json<Task>),
    #[oai(status = 404)]
    NotFound,
}

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/task", method = "get")]
    async fn list_task(&self, data: Data<&SqlitePool>) -> Result<Json<Vec<Task>>> {
        let tasks = sqlx::query_as!(
            Task,
            r#" 
            SELECT id, title, description, completed
            FROM tasks
            ORDER BY id
            "#,
        )
        .fetch_all(data.0)
        .await
        .map_err(InternalServerError)?;

        Ok(Json(tasks))
    }

    #[oai(path = "/task/:id", method = "get")]
    async fn get_task(&self, data: Data<&SqlitePool>, Path(id): Path<i64>) -> Result<TaskResponse> {
        let task = sqlx::query_as!(
            Task,
            r#" 
            SELECT id, title, description, completed
            FROM tasks
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(data.0)
        .await
        .map_err(InternalServerError)?;

        match task {
            Some(task) => Ok(TaskResponse::Ok(Json(task))),
            None => Ok(TaskResponse::NotFound),
        }
    }

    #[oai(path = "/task", method = "post")]
    async fn create_task(
        &self,
        data: Data<&SqlitePool>,
        Json(task): Json<Task>,
    ) -> Result<Json<Task>> {
        dbg!(&task);

        let task = sqlx::query_as!(
            Task,
            r#" 
            INSERT INTO tasks (title, description, completed)
            VALUES (?, ?, ?)
            RETURNING id, title, description, completed
            "#,
            task.title,
            task.description,
            task.completed
        )
        .fetch_one(data.0)
        .await
        .map_err(InternalServerError)?;

        Ok(Json(task))
    }

    #[oai(path = "/task/:id", method = "patch")]
    async fn update_task(
        &self,
        data: Data<&SqlitePool>,
        Path(id): Path<i64>,
        Json(task): Json<Task>,
    ) -> Result<TaskResponse> {
        let task = sqlx::query_as!(
            Task,
            r#" 
            UPDATE tasks
            SET title = ?, description = ?, completed = ? 
            WHERE id = ?
            RETURNING id as "id!", title, description, completed
            "#,
            task.title,
            task.description,
            task.completed,
            id
        )
        .fetch_optional(data.0)
        .await
        .map_err(InternalServerError)?;

        match task {
            Some(task) => Ok(TaskResponse::Ok(Json(task))),
            None => Ok(TaskResponse::NotFound),
        }
    }

    #[oai(path = "/task/:id", method = "delete")]
    async fn delete_task(
        &self,
        data: Data<&SqlitePool>,
        Path(id): Path<i64>,
    ) -> Result<TaskResponse> {
        let task = sqlx::query_as!(
            Task,
            r#" 
            DELETE FROM tasks
            WHERE id = ?
            RETURNING id, title, description, completed
            "#,
            id
        )
        .fetch_optional(data.0)
        .await
        .map_err(InternalServerError)?;

        match task {
            Some(task) => Ok(TaskResponse::Ok(Json(task))),
            None => Ok(TaskResponse::NotFound),
        }
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    dotenvy::dotenv().ok();
    color_eyre::install()?;

    // Setup tracing
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    // Setup database
    let url = std::env::var("DATABASE_URL")?;
    let pool = SqlitePool::connect(&url).await?;
    // sqlx::migrate!("./migrations").run(&pool).await?;

    // Setup & Start server
    let api_service = OpenApiService::new(Api, "TODO", "1.0").server("http://localhost:8000");
    let ui = api_service.openapi_explorer();
    let spec = api_service.spec_endpoint();

    let route = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .nest("/docs-json", spec)
        .data(pool)
        .with(middleware::Cors::default())
        .with(middleware::CatchPanic::default());
    Server::new(TcpListener::bind("127.0.0.1:8000"))
        .run(route)
        .await?;
    Ok(())
}
