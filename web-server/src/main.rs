//! A small annotated web server: an in-memory TODO API built with axum.
//!
//! Run:   cargo run
//! Try:   curl http://127.0.0.1:3030/
//!        curl -X POST http://127.0.0.1:3030/todos -H 'content-type: application/json' -d '{"title":"learn rust"}'
//!        curl http://127.0.0.1:3030/todos
//!        curl http://127.0.0.1:3030/todos?done=false
//!        curl http://127.0.0.1:3030/todos/1
//!        curl -X PATCH http://127.0.0.1:3030/todos/1 -H 'content-type: application/json' -d '{"done":true}'
//!        curl -X DELETE http://127.0.0.1:3030/todos/1
//!        curl -i http://127.0.0.1:3030/todos/99          (404 with a JSON error body)
//!
//! Lines starting with `//!` document the whole file (module docs).
//! Lines starting with `///` document the item right below them.
//! Plain `//` comments are ordinary comments.

// `use` brings names into scope so we can write `Router` instead of `axum::Router`.
// Curly braces import several names from the same path at once.
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// DATA TYPES
// ============================================================================

/// One TODO item.
///
/// `#[derive(...)]` asks the compiler to write trait implementations for us:
///   - `Debug`     -> lets us print it with `{:?}`
///   - `Clone`     -> lets us make an owned copy with `.clone()`
///   - `Serialize` -> (from serde) lets it be turned into JSON
///
/// A trait is like an interface: a set of methods a type promises to have.
#[derive(Debug, Clone, Serialize)]
struct Todo {
    id: u64,       // u64 = unsigned 64-bit integer (can never be negative)
    title: String, // String = owned, growable, heap-allocated text
    done: bool,
}

/// The JSON body expected by `POST /todos`, e.g. `{"title": "learn rust"}`.
///
/// `Deserialize` is the opposite of `Serialize`: it builds this struct from JSON.
/// If the body is missing `title` or it is not a string, axum rejects the
/// request with a 4xx before our handler runs, so the handler never sees bad data.
#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

/// The JSON body for `PATCH /todos/{id}`. Every field is optional, so
/// `{"done": true}` only changes `done` and leaves `title` alone.
///
/// `Option<T>` is Rust's replacement for null: it is either `Some(value)` or
/// `None`. A missing JSON field becomes `None`.
#[derive(Debug, Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    done: Option<bool>,
}

/// Query-string filters for `GET /todos?done=true`.
#[derive(Debug, Deserialize)]
struct ListFilter {
    done: Option<bool>,
}

// ============================================================================
// SHARED STATE
// ============================================================================

/// Everything the handlers share. In a real app this would hold a database
/// pool (like the `PgPool` in postgres-explorer); here it is a HashMap in memory.
#[derive(Default)] // Default -> `Db::default()` gives an empty map and next_id = 0
struct Db {
    todos: HashMap<u64, Todo>,
    next_id: u64,
}

/// The server handles many requests at the same time, on several threads,
/// and they all need the same `Db`. Rust will not compile code that lets two
/// threads change the same data without protection, so we wrap it:
///
///   - `Mutex<Db>`: only one thread at a time may hold the lock and touch the Db.
///   - `Arc<...>`: "Atomically Reference Counted" pointer. Cloning an Arc does
///     NOT copy the Db; it makes another pointer to the same Db and
///     bumps a counter. The Db is freed when the last Arc is dropped.
///
/// `type` creates an alias, so we can write `AppState` instead of the long type.
type AppState = Arc<Mutex<Db>>;

// ============================================================================
// ERRORS
// ============================================================================

/// The ways a request can fail. An `enum` is a type that is exactly one of
/// several variants, and a variant can carry data (`NotFound` carries the id).
#[derive(Debug)]
enum ApiError {
    NotFound(u64),
    EmptyTitle,
}

/// `IntoResponse` is axum's trait for "anything that can become an HTTP
/// response". Implementing it for `ApiError` lets handlers return
/// `Result<_, ApiError>`, and axum turns an `Err` into a proper response.
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // `match` must handle every variant. If we add a new variant to
        // ApiError later, this stops compiling until we handle it here too.
        let (status, message) = match self {
            ApiError::NotFound(id) => (StatusCode::NOT_FOUND, format!("todo {id} not found")),
            ApiError::EmptyTitle => (StatusCode::BAD_REQUEST, "title must not be empty".to_string()),
        };
        // `serde_json::json!` builds JSON inline. A tuple of (StatusCode, Json)
        // already implements IntoResponse, so we just convert it.
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// ============================================================================
// HANDLERS
// ============================================================================
//
// A handler is an ordinary `async fn`. Its parameters are "extractors": axum
// looks at each parameter's type and pulls that piece out of the request:
//   State(state)  -> the shared AppState
//   Path(id)      -> the `{id}` segment of the URL, parsed as u64
//   Query(filter) -> the `?done=true` query string
//   Json(body)    -> the request body, parsed as JSON
// `State(state)` in the parameter list is a pattern: it unwraps the State
// wrapper and binds the value inside to `state`.
//
// The return type only has to implement IntoResponse.

/// GET /  -> plain text
///
/// `&'static str` is a borrowed string slice that lives for the whole program
/// (string literals are baked into the binary).
async fn root() -> &'static str {
    "Hello from axum! Try GET /todos\n"
}

/// GET /todos  or  GET /todos?done=true
async fn list_todos(State(state): State<AppState>, Query(filter): Query<ListFilter>) -> Json<Vec<Todo>> {
    // `.lock()` waits for the mutex and returns a guard. The guard behaves like
    // `&mut Db`, and the lock is released automatically when `db` goes out of
    // scope at the end of the function. This is called RAII: no manual unlock.
    //
    // `.lock()` returns a Result because it fails if another thread panicked
    // while holding the lock. `.unwrap()` means "crash if that happened",
    // which is acceptable in a learning example.
    let db = state.lock().unwrap();

    // Iterators are lazy chains; nothing runs until `.collect()`.
    let mut todos: Vec<Todo> = db
        .todos
        .values() // iterate over &Todo (borrowed references, nothing copied yet)
        // `|t| ...` is a closure (an anonymous function).
        // `is_none_or`: no filter given -> keep everything,
        // otherwise keep the item only if `done` matches.
        .filter(|t| filter.done.is_none_or(|want| t.done == want))
        .cloned() // turn each &Todo into an owned Todo
        .collect(); // gather into the Vec<Todo> named in the type above

    // HashMap has no order, so sort by id to get stable output.
    todos.sort_by_key(|t| t.id);
    Json(todos)
}

/// GET /todos/{id}
async fn get_todo(State(state): State<AppState>, Path(id): Path<u64>) -> Result<Json<Todo>, ApiError> {
    let db = state.lock().unwrap();

    // `.get(&id)` returns Option<&Todo>.
    // `.cloned()`  turns it into Option<Todo>.
    // `.map(Json)` wraps the value in Json if there is one: Option<Json<Todo>>.
    // `.ok_or(..)` turns Option into Result: Some(v) -> Ok(v), None -> Err(..).
    db.todos.get(&id).cloned().map(Json).ok_or(ApiError::NotFound(id))
}

/// POST /todos   body: {"title": "..."}
async fn create_todo(
    State(state): State<AppState>,
    Json(input): Json<CreateTodo>, // the body extractor must be the last parameter
) -> Result<(StatusCode, Json<Todo>), ApiError> {
    // `.trim()` returns a &str that borrows from `input.title`; nothing is copied.
    let title = input.title.trim();
    if title.is_empty() {
        // An early return with an error. Validating before taking the lock
        // keeps the lock held for as short a time as possible.
        return Err(ApiError::EmptyTitle);
    }

    // `mut` because we are going to change the Db. Variables are immutable
    // unless you ask for `mut`.
    let mut db = state.lock().unwrap();
    db.next_id += 1;
    let todo = Todo {
        id: db.next_id,
        title: title.to_string(), // &str -> owned String
        done: false,
    };

    // `insert` takes ownership of what we pass in. We still need `todo` for
    // the response, so we insert a clone and keep the original.
    db.todos.insert(todo.id, todo.clone());

    // 201 Created plus the new item as JSON.
    Ok((StatusCode::CREATED, Json(todo)))
}

/// PATCH /todos/{id}   body: {"title": "...", "done": true}  (both optional)
async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateTodo>,
) -> Result<Json<Todo>, ApiError> {
    let mut db = state.lock().unwrap();

    // `get_mut` gives a mutable reference (&mut Todo) into the map, so we can
    // edit the stored item in place.
    // `?` is the error-propagation operator: on Err it returns that error from
    // this function right away; on Ok it unwraps the value and carries on.
    let todo = db.todos.get_mut(&id).ok_or(ApiError::NotFound(id))?;

    // `if let` runs the block only when the pattern matches, here when the
    // field was present in the JSON.
    if let Some(title) = input.title {
        let title = title.trim().to_string(); // shadowing: reuse the name for a new value
        if title.is_empty() {
            return Err(ApiError::EmptyTitle);
        }
        todo.title = title;
    }
    if let Some(done) = input.done {
        todo.done = done;
    }

    Ok(Json(todo.clone()))
}

/// DELETE /todos/{id}  -> 204 No Content
async fn delete_todo(State(state): State<AppState>, Path(id): Path<u64>) -> Result<StatusCode, ApiError> {
    let mut db = state.lock().unwrap();
    // `remove` returns the removed value as Option<Todo>. `match` on it:
    match db.todos.remove(&id) {
        Some(_) => Ok(StatusCode::NO_CONTENT), // `_` = "a value we don't need"
        None => Err(ApiError::NotFound(id)),
    }
}

// ============================================================================
// MAIN
// ============================================================================

/// `main` cannot be `async` by itself, because something has to run the
/// futures. `#[tokio::main]` is a macro that starts the tokio runtime (a
/// thread pool that drives async tasks) and runs our async main inside it.
#[tokio::main]
async fn main() {
    let state: AppState = Arc::new(Mutex::new(Db::default()));

    // The Router maps (path, HTTP method) pairs to handlers. Method calls are
    // chained: each `.route(...)` takes the router and returns a new one.
    // `{id}` is a path parameter, read by the `Path` extractor.
    let app = Router::new()
        .route("/", get(root))
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/{id}", get(get_todo).patch(update_todo).delete(delete_todo))
        // Hand the state to the router. Each request gets a clone of the Arc
        // (a cheap pointer copy), not a copy of the data.
        .with_state(state);

    let addr = "127.0.0.1:3030";

    // `.await` pauses this task until the future finishes, without blocking the
    // thread. While we wait, the thread can work on other tasks.
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("could not bind; is something else using port 3030?");

    println!("listening on http://{addr}");

    // Serve forever. `axum::serve` returns a Result; `.unwrap()` crashes with
    // the error if the server ever fails.
    axum::serve(listener, app).await.unwrap();
}
