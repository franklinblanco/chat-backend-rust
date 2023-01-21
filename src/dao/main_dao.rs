use std::collections::HashMap;

use sqlx::MySqlPool;

#[allow(dead_code)]
pub async fn start_database_connection(
    env_vars: &HashMap<String, String>,
) -> Result<MySqlPool, sqlx::Error> {
    let db_url = match env_vars.get("DATABASE_URL") {
        Some(str) => str,
        None => panic!("DATABASE_URL env var not found"),
    };
    let formatted_db_url = &db_url;
    sqlx::MySqlPool::connect(&formatted_db_url).await
}
#[allow(dead_code)]

pub async fn run_all_migrations(conn: &MySqlPool) {
    match sqlx::migrate!("./migrations").run(conn).await {
        Ok(()) => {
            println!("{}", "Successfully ran migrations.")
        }
        Err(error) => {
            panic!("{error}")
        }
    }
}
