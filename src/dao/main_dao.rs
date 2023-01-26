use std::{collections::HashMap, env};

use sqlx::MySqlPool;

#[allow(dead_code)]
pub async fn start_database_connection() -> Result<MySqlPool, sqlx::Error> {
    let vars = env::vars().into_iter().collect::<HashMap<String, String>>();
    let db_url = match vars.get("DATABASE_URL") {
        Some(str) => str,
        None => panic!("DATABASE_URL env var not found"),
    };
    let formatted_db_url = &db_url;
    sqlx::MySqlPool::connect(formatted_db_url).await
}
#[allow(dead_code)]
pub async fn run_all_migrations(conn: &MySqlPool) {
    match sqlx::migrate!("./migrations").run(conn).await {
        Ok(()) => {
            println!("Successfully ran migrations.")
        }
        Err(error) => {
            panic!("{error}")
        }
    }
}
