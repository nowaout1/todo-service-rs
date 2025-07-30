use todo_service::{TodoService, config::Config, todo::todo_server::TodoServer};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Config { db_url, addr } = Config::parse().await?;
    let todo_service = TodoService::init(&db_url).await?;

    Server::builder()
        .add_service(TodoServer::new(todo_service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
