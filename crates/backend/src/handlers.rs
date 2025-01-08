use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response}, Json,
};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::info;

use model::ImageData;

use crate::{AppState, add_first_count, add_second_count};

const PATH_PREFIX_IMAGE: &str = "images";
const PATH_PREFIX_THUMB: &str = "thumbs";

pub async fn greet_handler() -> Html<&'static str> {
    Html("<h1>Nice to meet you!</h1>")
}

pub async fn upload_handler(mut multipart: Multipart) -> Result<Json<Vec<String>>, Response<Body>> {
    let mut filenames = Vec::new();
    while let Ok(Some(field)) = multipart.next_field().await {
        let field_name = field.name().unwrap_or("unnamed").to_owned();
        if [PATH_PREFIX_IMAGE, PATH_PREFIX_THUMB].contains(&field_name.as_str()) {
            let field_filename = field.file_name().unwrap_or("no_filename").to_owned();
            let field_content_type = field.content_type().unwrap_or("no_content_type").to_owned();
            let data = match field.bytes().await {
                Ok(data) => data,
                Err(e) => {
                    return Err(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!("Failed to read data for field '{}': {}", field_name, e)))
                        .unwrap());
                }
            };
            let file_path = ["volume", &field_name, &field_filename].join("/");
            let current = std::env::current_dir().unwrap();
            let path = current.join(file_path);
            let prefix = path.parent().unwrap();
            tokio::fs::create_dir_all(prefix).await.unwrap();
            let mut f = File::create(&path).await.unwrap();
            f.write_all(&data).await.unwrap();
            info!("Received field: {} {} {} ({} bytes)", &field_name, &field_filename, &field_content_type, data.len());
            if field_name.as_str() == PATH_PREFIX_THUMB {
                filenames.push(field_filename);
            }
        }
    }
    Ok(Json(filenames))
}

pub async fn get_first(Path(foreign_id): Path<u32>, State(app): State<AppState>) -> impl IntoResponse {
    if let Ok(lock) = app.first_table.lock() {
        let result = lock.iter().filter_map(|(_, data)| {
            if data.foreign_id == foreign_id {
                Some(data.clone())
            } else {
                None
            }
        })
        .collect::<Vec<ImageData>>();

        (StatusCode::OK, Json(result))
    } else {
        (StatusCode::NOT_FOUND, Json(Vec::new()))
    }
}

pub async fn get_second(Path(foreign_id): Path<u32>, State(app): State<AppState>) -> impl IntoResponse {
    if let Ok(lock) = app.second_table.lock() {
        let result = lock.iter().filter_map(|(_, data)| {
            if data.foreign_id == foreign_id {
                Some(data.clone())
            } else {
                None
            }
        })
        .collect::<Vec<ImageData>>();

        (StatusCode::OK, Json(result))
    } else {
        (StatusCode::NOT_FOUND, Json(Vec::new()))
    }
}

pub async fn post_first(
    State(app): State<AppState>,
    Json(payloads): Json<Vec<ImageData>>,
) -> impl IntoResponse {
    if let Ok(mut lock) = app.first_table.lock() {
        for mut payload in payloads {
            let id = add_first_count();
            payload.image_id = id;
            let _ = lock.insert(id, payload);
        }

        (StatusCode::OK, Json::<Vec<String>>(Vec::new()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
    }
}

pub async fn post_second(
    State(app): State<AppState>,
    Json(payloads): Json<Vec<ImageData>>,
) -> impl IntoResponse {
    if let Ok(mut lock) = app.second_table.lock() {
        for mut payload in payloads {
            let id = add_second_count();
            payload.image_id = id;
            let _ = lock.insert(id, payload);
        }

        (StatusCode::OK, Json::<Vec<String>>(Vec::new()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
    }
}