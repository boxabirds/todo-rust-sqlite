use actix_files as fs;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
struct Todo {
    id: Option<i32>,  // Changed to Option<i32> because ID may not be present in the incoming JSON
    task: String,
    done: bool,
}

struct AppState {
    conn: Mutex<Connection>,
}

async fn get_todos(data: web::Data<AppState>) -> impl Responder {
    let conn = data.conn.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, task, done FROM todos").unwrap();
    let todos_iter = stmt.query_map([], |row| {
        Ok(Todo {
            id: Some(row.get(0)?),
            task: row.get(1)?,
            done: row.get(2)?,
        })
    }).unwrap();

    let todos: Vec<Todo> = todos_iter.map(|todo| todo.unwrap()).collect();
    HttpResponse::Ok().json(todos)
}

async fn add_todo(data: web::Data<AppState>, new_todo: web::Json<Todo>) -> impl Responder {
    println!("Received JSON: {:?}", new_todo);  // Debug log
    let conn = data.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO todos (task, done) VALUES (?1, ?2)",
        params![new_todo.task, new_todo.done],
    ).unwrap();
    HttpResponse::Created().finish()
}

async fn remove_todo(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let conn = data.conn.lock().unwrap();
    conn.execute("DELETE FROM todos WHERE id = ?1", params![*id]).unwrap();
    HttpResponse::Ok().finish()
}

fn init_db() -> Connection {
    let conn = Connection::open("todos.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task TEXT NOT NULL,
            done BOOLEAN NOT NULL
        )",
        [],
    ).unwrap();
    conn
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let conn = init_db();
    let data = web::Data::new(AppState {
        conn: Mutex::new(conn),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(
                web::resource("/todos")
                    .route(web::get().to(get_todos))
            )
            .service(
                web::resource("/addTodo")
                    .route(web::post().to(add_todo))
            )
            .service(
                web::resource("/removeTodo/{id}")
                    .route(web::delete().to(remove_todo))
            )
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
