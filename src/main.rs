
use axum::{                                                                                                                                                                      
    extract::{self, Path, Query},  
    routing::{get, post},                                                                                                                                                        
    Json, Router,                        
};       
use minio_rsc::{Minio, provider::StaticProvider, client::PresignedArgs};
use serde::{Deserialize, Serialize};                                                                                                                                                          
use serde_json::{json, Value};                                                                                                                                                  
use sqlx::PgPool;                                                                                                                                                               
use sqlx::{postgres::PgPoolOptions, prelude::FromRow};                                                                                                                           
use std::env;                                                                                                                                                                    
use std::net::SocketAddr;                                                                                                                                                        
use std::result::Result;                                                                                                                                                         
use std::sync::Arc;                                                                                                                                                              
use axum::http::StatusCode;                  
use sqlx::types::chrono::Utc; 
use std::collections::HashMap;
use tower_http::cors::{AllowOrigin, CorsLayer};
use axum::http::Method;
use reqwest;


#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Users {
    user_id: Option<uuid::Uuid>,
    email: String,
    name: String,
}

pub async fn add_users(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<Users>,
) -> Json<Value> {
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_add_users(extract::State(pool), Json(payload)).await;
    result
}

pub async fn data_add_users(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<Users>,
) -> Json<Value> {
    let query = "INSERT INTO users (email, name) VALUES ($1, $2) RETURNING *";
    
    let q = sqlx::query_as::<_, Users>(&query)
		.bind(payload.email)
		.bind(payload.name);
    
    let result = q.fetch_one(&pool).await;

    match result {
        Ok(value) => Json(json!({"res": "success", "data": value})),
        Err(e) => Json(json!({"res": format!("error: {}", e)}))
    }
}


#[derive(Deserialize)]
struct usersQueryParams {
    order_by: Option<String>,
    direction: Option<String>, // "asc" or "desc"
    #[serde(flatten)]
    filters: HashMap<String, String>,
}


pub async fn get_users(
    extract::State(pool): extract::State<PgPool>,
    Query(query_params): Query<usersQueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_get_users(extract::State(pool), axum::extract::Query(query_params)).await;
    result
}



pub async fn data_get_users(
    extract::State(pool): extract::State<PgPool>,
    query_params: axum::extract::Query<usersQueryParams>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let mut query = "SELECT * FROM users".to_owned();
    let mut sql_params: Vec<String> = Vec::new();
    let mut param_index = 1;
    
    // Handle filters
    if !query_params.filters.is_empty() {
        let mut where_conditions: Vec<String> = Vec::new();
        
        for (field, value) in &query_params.filters {
            // Skip ordering parameters
            if field == "order_by" || field == "direction" {
                continue;
            }
            
            // Validate field name to prevent SQL injection
            if field.chars().all(|c| c.is_alphanumeric() || c == '_') {
                where_conditions.push(format!("{} = ${}", field, param_index));
                sql_params.push(value.clone());
                param_index += 1;
            } else {
                return Err((StatusCode::BAD_REQUEST, format!("Invalid field name: {}", field)));
            }
        }
        
        if !where_conditions.is_empty() {
            query.push_str(&(" WHERE ".to_owned() + &where_conditions.join(" AND ")));
        }
    }
    
    // Validate and apply ordering if provided
    if let Some(order_by) = &query_params.order_by {
        // Validate order_by column name to prevent SQL injection
        // Only allow alphanumeric characters and underscores
        if order_by.chars().all(|c| c.is_alphanumeric() || c == '_') {
            // Validate direction parameter
            let direction = match &query_params.direction {
                Some(dir) if dir.to_lowercase() == "desc" => "DESC",
                _ => "ASC",
            };
            
            query.push_str(&format!(" ORDER BY {} {}", *order_by, direction));
        } else {
            return Err((StatusCode::BAD_REQUEST, "Invalid order_by parameter".to_string()));
        }
    }

    // Execute query with parameters
    let mut query_builder = sqlx::query_as::<_, Users>(&query);
    for param in &sql_params {
        query_builder = query_builder.bind(param);
    }

    let elemints: Vec<Users> = query_builder.fetch_all(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let res_json: Vec<Value> = elemints.into_iter().map(|elemint| {
        json!({
    	"user_id": elemint.user_id, 
	"email": elemint.email, 
	"name": elemint.name
        })
    }).collect();

    Ok(Json(json!({ "payload": res_json })))
}

#[derive(Debug, Deserialize)]
struct usersuser_idQuery {
    user_id: uuid::Uuid,
}

pub async fn get_one_usersuser_id(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<usersuser_idQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_get_one_usersuser_id(extract::State(pool), match_val).await;
    result
}

pub async fn data_get_one_usersuser_id(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<usersuser_idQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let query = format!("SELECT * FROM users WHERE user_id = $1");
    let q = sqlx::query_as::<_, Users>(&query).bind(match_val.user_id.clone());

    let elemint = q.fetch_optional(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database err{}", e))
    })?;

    match elemint {
        Some(elemint) => Ok(Json(json!({
            "payload": {
                	"user_id": elemint.user_id, 
	"email": elemint.email, 
	"name": elemint.name, 

            }
        }))),
        None => Err((StatusCode::NOT_FOUND, format!("No record found with user_id = the value"))),
    }
}



#[derive(Debug, Deserialize)]
struct usersemailQuery {
    email: String,
}

pub async fn get_one_usersemail(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<usersemailQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_get_one_usersemail(extract::State(pool), match_val).await;
    result
}

pub async fn data_get_one_usersemail(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<usersemailQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let query = format!("SELECT * FROM users WHERE email = $1");
    let q = sqlx::query_as::<_, Users>(&query).bind(match_val.email.clone());

    let elemint = q.fetch_optional(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database err{}", e))
    })?;

    match elemint {
        Some(elemint) => Ok(Json(json!({
            "payload": {
                	"user_id": elemint.user_id, 
	"email": elemint.email, 
	"name": elemint.name, 

            }
        }))),
        None => Err((StatusCode::NOT_FOUND, format!("No record found with email = the value"))),
    }
}



#[derive(Debug, Deserialize)]
struct usersnameQuery {
    name: String,
}

pub async fn get_one_usersname(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<usersnameQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Call data function from data module 
    // Other business logic can also be handled here 
    let result = data_get_one_usersname(extract::State(pool), match_val).await;
    result
}

pub async fn data_get_one_usersname(
    extract::State(pool): extract::State<PgPool>,
    match_val: Query<usersnameQuery>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let query = format!("SELECT * FROM users WHERE name = $1");
    let q = sqlx::query_as::<_, Users>(&query).bind(match_val.name.clone());

    let elemint = q.fetch_optional(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database err{}", e))
    })?;

    match elemint {
        Some(elemint) => Ok(Json(json!({
            "payload": {
                	"user_id": elemint.user_id, 
	"email": elemint.email, 
	"name": elemint.name, 

            }
        }))),
        None => Err((StatusCode::NOT_FOUND, format!("No record found with name = the value"))),
    }
}





async fn python() -> Result<Json<Value>, (StatusCode, String)> {
    // Call the Python FastAPI service
    let mut map = HashMap::new();// maybe user json! insted of map?? 
    map.insert("message", "what is penn");
    let client = reqwest::Client::new();
    let res = client
        .post("http://python:8003/chat")  // Use service name and correct port
        .json(&map)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Request failed: {}", e)))?;

    if res.status().is_client_error() || res.status().is_server_error() {
        return Err((StatusCode::BAD_REQUEST, format!("Error from Python service: {}", res.status())));
    }

    let json_response: Value = res
        .json()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse JSON: {}", e)))?;

    Ok(Json(json!({"payload": json_response})))
}


async fn health() -> String {"healthy".to_string() }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = env::var("DATABASE_URL")
     .unwrap_or_else(|_| "postgres://dbuser:p@localhost:1111/data".to_string());
    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&db_url)
        .await?;

    let migrate = sqlx::migrate!("./migrations").run(&pool).await;

    match migrate {
        Ok(_) => println!("Migrations applied successfully."),
        Err(e) => eprintln!("Error applying migrations: {}", e),
    };

    let app = Router::new()
    .route("/health", get(health))
    	.route("/add_users", post(add_users))
	.route("/get_users", get(get_users))
	.route("/get_one_usersuser_id", get(get_one_usersuser_id))
	.route("/get_one_usersemail", get(get_one_usersemail))
	.route("/get_one_usersname", get(get_one_usersname))
	.route("/signed-urls/:video_path", get(get_signed_url))

    .route("/python", post(python))
    .layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(vec![
                "http://localhost:3000".parse().unwrap(),
                "https://example.com".parse().unwrap(),
            ]))
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(tower_http::cors::Any)
    )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}



async fn generate_signed_url(object_key: String) -> Result<String, anyhow::Error> {
    let endpoint = env::var("MINIO_ENDPOINT")
        .unwrap_or_else(|_| "localhost:9001".to_string());
    let access_key = env::var("MINIO_ACCESS_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let secret_key = env::var("MINIO_SECRET_KEY").unwrap_or_else(|_| "minioadmin".to_string());
    let bucket = env::var("MINIO_BUCKET").unwrap_or_else(|_| "bucket".to_string());
    let endpoint = env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "localhost:9000".to_string());
    let secure = env::var("MINIO_SECURE")
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false);

    let provider = StaticProvider::new(&access_key, &secret_key, None);

    let minio = Minio::builder()
        .endpoint(&endpoint)
        .provider(provider)
        .secure(secure)
        .region("us-east-1".to_string())  // Explicitly set region to match MinIO default
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create MinIO client: {}", e))?;

    let presigned_url = minio
        .presigned_get_object(
            PresignedArgs::new(bucket, object_key)
                .expires(3600),  // 1 hour in seconds
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to generate presigned URL: {}", e))?;
    Ok(presigned_url)
}
    
use axum::response::IntoResponse;

async fn get_signed_url(
    Path(video_path): Path<String>,
) -> impl IntoResponse {
    let object_key = video_path; 
    println!("Environment variables:");
    println!("MINIO_ENDPOINT: {}", env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "not set".to_string()));
    println!("MINIO_BUCKET: {}", env::var("MINIO_BUCKET").unwrap_or_else(|_| "not set, using default 'test'".to_string()));
    
    match generate_signed_url(object_key).await {
        Ok(url) => (StatusCode::OK, url).into_response(),
        Err(e) => {
            eprintln!("Error generating signed URL: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to generate signed URL: {}", e)).into_response()
        }
    }
}
async fn upload_video(
    // mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, String)> {
    let provider = StaticProvider::new("minioadmin", "minioadmin", None);
    let minio = Minio::builder()
        .endpoint("minio:9000")
        .provider(provider)
        .secure(false)
        .build()
        .unwrap();

        let _data = "hello minio";

        let upload_result = minio.put_object("bucket", "file.txt", _data.into()).await;
        
        return Ok(Json(json!({
            "status": upload_result.is_ok(),
            "message": if upload_result.is_ok() {
                "File uploaded successfully"
            } else {
                "Failed to upload file"
            },
            "file_name": "file.txt"
        })));
}
    
