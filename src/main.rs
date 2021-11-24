mod database;
mod models;
use crate::database::Database;
use anyhow::Result;
use axum::{extract::Extension, http::StatusCode, routing::get, AddExtensionLayer, Json, Router};
use dotenv::dotenv;
use models::{GetResponse, PostRequest};
use std::path::Path;

static SETUP_MODE: bool = false;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse config from environment
    dotenv().ok();
    let config = envy::from_env()?;

    let mut db = Database::connect(&config).await?;

    if SETUP_MODE {
        return db.setup(Path::new(&config.setup_csv.unwrap())).await;
    }

    let app = Router::new()
        .route("/task", get(get_task).post(submit_task))
        .layer(AddExtensionLayer::new(db))
        .layer(tower_http::auth::RequireAuthorizationLayer::basic(
            "bot",
            &config.listen_password,
        ));

    println!("listening on {}", &config.listen_addr);
    axum::Server::bind(&config.listen_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn get_task(
    Extension(mut db): Extension<Database>,
) -> Result<Json<GetResponse>, (StatusCode, String)> {
    let response = db.get_datapoint().await;
    match response {
        Ok((data, id)) => Ok(Json::from(GetResponse { id, data })),
        Err(err) => Err((StatusCode::BAD_REQUEST, err.to_string())),
    }
}

async fn submit_task(
    Json(request): Json<PostRequest>,
    Extension(mut db): Extension<Database>,
) -> (StatusCode, String) {
    let PostRequest { id: row, data } = request;
    let response = db.set_datapoint(row, data).await;
    match response {
        Ok(_) => (StatusCode::OK, "Submitted!".to_string()),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()),
    }
}
