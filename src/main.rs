mod db;
mod structures;

use std::fmt::format;
use crate::db::db_model;
use crate::structures::Order;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::net::SocketAddr;
use std::sync::{Arc};
use clap::Parser;
use log::{debug, error, info, warn};
use tokio::sync::RwLock;
use tokio_postgres::Client;

//Структура, хранящая аргументы командной строки, необходимые для запуска сервера и подключения к базе данных PostgreSQL
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args{
    #[arg(long)]
    server_host: String,

    #[arg(long)]
    server_port: i32,

    #[arg(long)]
    db_type: String,

    #[arg(long)]
    db_host: String,

    #[arg(long)]
    db_user: String,

    #[arg(long)]
    db_password: String,

    #[arg(long)]
    db_name: String,

    #[arg(long)]
    db_port: i32
}

//Структура для передачи состояния при работе сервера
struct AppState {
    client: Client,
    db_model: db_model,
}

#[tokio::main]
pub async fn main() {
    //Инициализация логера
    log4rs::init_file("log4.yaml", Default::default()).unwrap();

    //Считывание аргументов командной строки
    let args = Args::parse();
    info!("Init args");

    //Создание новой структуры, содержащей данные для подключения к базе данных
    let db = db_model::new(
        args.db_type,
        args.db_host,
        args.db_user,
        args.db_password,
        args.db_name,
        args.db_port.to_string()
    );

    //Переменная, через которую осуществляется обращение к базе данных, инициализация базы данных
    let client = db.init().await;
    info!("DataBase connection was successful");

    let state = Arc::new(RwLock::new(AppState {
        client: client,
        db_model: db,
    }));

    //Настройка маршрутизатора сервера
    let router: Router = Router::new()
        .route("/get", get(get_handler))
        .route("/add", post(post_handler))
        .fallback(fallback_handler)
        .with_state(state);

    let addr:SocketAddr = format!("{}:{}", args.server_host, args.server_port).parse().unwrap();
    info!("Starts server on {}", addr);

    //Запуск сервера по адресу, хранящемуся в переменной addr
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

//Обработчик GET-запроса по адресу /get. Принимает состояние, возвращает ответ с json в случае успешного извлечения данных из бд
async fn get_handler(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let db = state.read().await;
    let res = db.db_model.get_orders(&db.client).await;
    match res {
        Ok(vec) => {
            let json_orders = serde_json::to_string(&vec).unwrap();
            info!("Orders list for get-request: {}", json_orders);
            (StatusCode::OK, json_orders)
        }
        Err(e) => {
            error!("Err on get-request: {}", e.to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, String::from("Couldn't get the orders"))},
    }
}

//Обработчик POST-запроса по адресу /add. Принимает состояние, json со структурой заказа, сообщение об успехе операции
pub async fn post_handler(State(state): State<Arc<RwLock<AppState>>>, Json(order): Json<Order>,) -> impl IntoResponse {
    let mut db = state.write().await;
    debug!("Received JSON: {}", serde_json::to_string(&order).unwrap());
    match db.db_model.add_order(order, &db.client).await {
        Ok(_) => {
            info!("JSON added successfully");
            (StatusCode::OK, "Order was successfully saved")},
        Err(e) => {
            error!("Err while adding JSON: {}", e.to_string());
        (StatusCode::INTERNAL_SERVER_ERROR, "Couldn't save the order")},
    }
}

//Обработчик некорректных запросов, возвращает сообщение
pub async fn fallback_handler() -> impl IntoResponse {
    warn!("Incorrect url");
    (
        StatusCode::NOT_FOUND,
        String::from("There's nothing in this address ;)\n"),
    )
}
