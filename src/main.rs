use askama::Template;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, patch, post},
    Form, Router,
};
use lazy_static::lazy_static;
use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct TaskStore {
    desc: String,
    done: bool,
}


lazy_static! {
    static ref TASKS: Mutex<Vec<TaskStore>> = Mutex::new(vec![]);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "with_axum_htmx_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Hello, web server!");

    let app = Router::new()
        .route("/", get(show_tasks))
        .route("/add", post(add_task))
        .route("/delete/:id", delete(delete_task))
        .route("/check/:id", patch(check_task));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

struct Task {
    desc: String,
    done: bool,
    id: usize,
}

fn add_index_to_tasks(tasks: Vec<TaskStore>) -> Vec<Task> {
    tasks
        .iter()
        .enumerate()
        .map(|(index, task)| Task {
            desc: task.desc.to_string(),
            done: task.done,
            id: index,
        })
        .collect()
}

#[derive(Template)]
#[template(path = "tasks.html")]
struct TasksTemplate<'a> {
    tasks: &'a Vec<Task>,
}

async fn show_tasks() -> impl IntoResponse {
    let tasks = TASKS.lock().unwrap();
    let template = TasksTemplate {
        tasks: &add_index_to_tasks(tasks.to_vec()),
    };
    HtmlTemplate(template).into_response()
}

#[derive(Template)]
#[template(path = "tasks-list.html")]
struct TasksListTemplate<'a> {
    tasks: &'a Vec<Task>,
}

#[derive(serde::Deserialize)]
struct AddTask {
    task: String,
}

async fn add_task(Form(input): Form<AddTask>) -> impl IntoResponse {
    let mut tasks = TASKS.lock().unwrap();

    if input.task.len() > 0 {
        tasks.push(TaskStore {
            desc: input.task,
            done: false,
        });
    }

    // TODO : if not added, send feedback

    HtmlTemplate(TasksListTemplate {
        tasks: &add_index_to_tasks(tasks.to_vec()),
    })
    .into_response()
}

async fn delete_task(Path(id): Path<usize>) -> impl IntoResponse {
    let mut tasks = TASKS.lock().unwrap();

    tasks.remove(id);

    HtmlTemplate(TasksListTemplate {
        tasks: &add_index_to_tasks(tasks.to_vec()),
    })
    .into_response()
}

async fn check_task(Path(id): Path<usize>) -> impl IntoResponse {
    let mut tasks = TASKS.lock().unwrap();
    let task = tasks.get_mut(id);

    if let Some(task) = task {
        task.done = true;
    }

    HtmlTemplate(TasksListTemplate {
        tasks: &add_index_to_tasks(tasks.to_vec()),
    })
    .into_response()
}
