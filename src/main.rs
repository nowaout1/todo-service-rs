use tonic::transport::Server;

use todo_service::config::Config;
use todo_service_api::{TodoServiceApi, todo_proto::todo_server::TodoServer};
use todo_service_data::db::postgres::TodoPostgres;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let Config { db_url, addr } = Config::parse().await?;

    let todo_repository = TodoPostgres::init(&db_url).await?;
    let todo_service = TodoServiceApi::new(todo_repository);

    Server::builder()
        .add_service(TodoServer::new(todo_service))
        .serve(addr.parse()?)
        .await?;

    Ok(())
}
