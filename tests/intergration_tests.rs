use tonic::transport::{Channel, Server};

use todo_service::config::Config;
use todo_service_api::{
    TodoServiceApi,
    todo_proto::{self, todo_client, todo_server},
};
use todo_service_data::db::postgres::TodoPostgres;

async fn start_test_server() -> todo_client::TodoClient<Channel> {
    let Config { db_url, .. } = Config::parse().await.unwrap();

    let todo_repository = TodoPostgres::init(&db_url).await.unwrap();
    let todo_service = TodoServiceApi::new(todo_repository);

    tokio::spawn(async move {
        Server::builder()
            .add_service(todo_server::TodoServer::new(todo_service))
            .serve("[::1]:50051".parse().unwrap())
            .await
            .unwrap();
    });

    todo_client::TodoClient::connect("http://[::1]:50051")
        .await
        .unwrap()
}

#[tokio::test]
async fn fetch_by_non_existing_id() {
    let mut client = start_test_server().await;

    let id = uuid::Uuid::now_v7().to_string();
    let res = client
        .fetch_todo_by_id(todo_proto::FetchTodoByIdRequest { id })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(res.todo, None);
}

#[tokio::test]
async fn fetch_todos_by_range_assert_count() {
    let mut client = start_test_server().await;

    let res = client
        .fetch_todos_by_range(todo_proto::FetchTodosByRangeRequest {
            offset: 0,
            limit: 10,
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(res.todos.len(), res.count as usize);
}

#[tokio::test]
async fn create_todo() {
    let mut client = start_test_server().await;

    let _ = client
        .create_todo(todo_proto::CreateTodoRequest {
            title: "Create todo".to_owned(),
            description: None,
        })
        .await;
}

#[tokio::test]
async fn update_non_existing_todo() {
    let mut client = start_test_server().await;

    let id = uuid::Uuid::now_v7().to_string();
    let res = client
        .update_todo(todo_proto::UpdateTodoRequest {
            id,
            title: Some("New title".to_owned()),
            description: None,
            is_done: None,
        })
        .await;

    assert!(res.is_err());
}

#[tokio::test]
async fn delete_non_existing_todo() {
    let mut client = start_test_server().await;

    let id = uuid::Uuid::now_v7().to_string();
    let res = client
        .fetch_todo_by_id(todo_proto::FetchTodoByIdRequest { id })
        .await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn crud_todo() {
    let mut client = start_test_server().await;

    // Create
    let id = client
        .create_todo(todo_proto::CreateTodoRequest {
            title: "Create todo".to_owned(),
            description: None,
        })
        .await
        .unwrap()
        .into_inner()
        .todo
        .unwrap()
        .id;

    // Update
    let id = client
        .update_todo(todo_proto::UpdateTodoRequest {
            id,
            title: Some("Updated todo".to_owned()),
            description: None,
            is_done: Some(true),
        })
        .await
        .unwrap()
        .into_inner()
        .todo
        .unwrap()
        .id;

    // Read
    let id = client
        .fetch_todo_by_id(todo_proto::FetchTodoByIdRequest { id })
        .await
        .unwrap()
        .into_inner()
        .todo
        .unwrap()
        .id;

    // Delete
    let _ = client
        .delete_todo(todo_proto::DeleteTodoRequest { id })
        .await
        .unwrap();
}
