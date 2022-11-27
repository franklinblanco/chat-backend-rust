use surrealdb_rs::{Surreal, protocol::Ws, param::Root, net::WsClient};


pub async fn connect_to_db() -> Result<Surreal<WsClient>, surrealdb_rs::Error> {
    let client = Surreal::connect::<Ws>("127.0.0.1:8000").await?;

    // Signin as a namespace, database, or root user
    client.signin(Root {
        username: "root",
        password: "root",
    }).await?;

    // Select a specific namespace / database
    client.use_ns("test").use_db("test").await?;
    Ok(client)
}