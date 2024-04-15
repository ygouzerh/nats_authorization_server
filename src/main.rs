use axum::{
    extract::Path, http::StatusCode, response::IntoResponse, routing::get, Extension, Router
};

use std::{env, net::{IpAddr, SocketAddr}, str::FromStr, sync::Arc};
use tokio_postgres::NoTls;

async fn setup_postgres_client() -> tokio_postgres::Client {
    let database_connection_string = env::var("AUTHORIZATION_DB_CONNECTION_STRING").expect("AUTHORIZATION_DB_CONNECTION_STRING must be set");
    let (postgres_client, connection) =
        tokio_postgres::connect(&database_connection_string, NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    postgres_client

}

async fn get_account_key_per_id(postgres_client: Arc<tokio_postgres::Client>, account_id: &str) -> Result<String, String> {
    let rows = postgres_client.query("SELECT account_jwt FROM nats WHERE nsc_account_id = $1", &[&account_id])
        .await
        .map_err(|err| format!("Failed to run query: {}", err));

    let rows = rows.unwrap();
    let row = rows.get(0);
    if let None = row {
        return Err("No rows found".to_string());
    }
    let account_jwt: String = row.unwrap().get(0);
    Ok(account_jwt)
    
}

async fn account_details(
    Extension(postgres_client): Extension<Arc<tokio_postgres::Client>>,
    Path(account_id): Path<String>
) -> impl IntoResponse {
    let account_jwt = get_account_key_per_id(postgres_client, &account_id).await;
    match account_jwt {
        Ok(account_jwt) => (StatusCode::OK, account_jwt),
        Err(_) => (StatusCode::NOT_FOUND, "Account not found".to_string())
    }
}

async fn accounts_base() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[tokio::main]
async fn main() {

    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);

    let host = env::var("AUTHORIZATION_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("AUTHORIZATION_PORT").unwrap_or_else(|_| "9091".to_string());
    
    // Set up the router
    let app = Router::new()
        .route("/jwt/v1/accounts/:account_id", get(account_details)) // Dynamic segment
        .route("/jwt/v1/accounts/", get(accounts_base))
        .layer(Extension(Arc::clone(&postgres_client))); // Base path
    
    // Define the server address
    let ip_addr = IpAddr::from_str(&host).unwrap();
    let port_num = port.parse::<u16>().unwrap();
    let addr = SocketAddr::new(ip_addr, port_num);
    
    println!("Listening on {}", addr);

    // Start the server
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_get_account_jwt() {
        // Setup of the test
        let postgres_client = super::setup_postgres_client().await;
        let postgres_client = Arc::new(postgres_client);
        let postgres_client_two = Arc::clone(&postgres_client);

        // The user_id must be an existing one in the user table
        let user_id = Uuid::parse_str("7c278ecc-d624-45a0-aa87-9add7253b517").unwrap();
        let nsc_account_id = "nsc_account_id_dummy";
        let creds_admin = "creds_admin_dummy";
        let creds_user = "creds_user_dummy";
        let account_jwt = "account_jwt_dummy";
    
        let _ = postgres_client_two.execute("DELETE FROM nats WHERE id = $1", &[&user_id])
            .await;
    
        let result = postgres_client.execute("INSERT INTO nats (id, nsc_account_id, creds_admin, creds_user, account_jwt) VALUES ($1, $2, $3, $4, $5)", &[&user_id, &nsc_account_id, &creds_admin, &creds_user, &account_jwt])
            .await;
        assert!(result.is_ok(), "Failed to insert nsc user: {:?}", result);
        
        let result = tokio::spawn(async move {
            // Test
            let result = super::get_account_key_per_id(Arc::clone(&postgres_client), nsc_account_id).await;
            assert!(result.is_ok(), "Failed to get account jwt: {:?}", result);
            let account_jwt = result.unwrap();
            assert!(!account_jwt.is_empty(), "Account JWT should not be empty");
            assert!(account_jwt == account_jwt, "Account JWT should be equal to {}", account_jwt);
        }).await;

        assert!(result.is_ok(), "Failed the test to get account jwt: {:?}", result);
    
        // Cleanup
        // let result = postgres_client_two.execute("DELETE FROM nats WHERE id = $1", &[&user_id])
        //     .await;
        // assert!(result.is_ok(), "Failed to delete nsc user: {:?}", result);
    }
}