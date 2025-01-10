use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response}, Json,
};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::info;

use model::ImageData;

use crate::{AppState, add_count};

const PATH_PREFIX_IMAGE: &str = "images";
const PATH_PREFIX_THUMB: &str = "thumbs";

pub async fn greet_handler() -> Html<&'static str> {
    Html("<h1>Nice to meet you!</h1>")
}

pub async fn post_image(
    State(app): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Vec<ImageData>>, Response<Body>> {
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
                let image = ImageData {
                    image_id: add_count(),
                    foreign_id: 0,
                    path: field_filename.clone(),
                    title: None,
                    user: String::from("user"),
                };
                {
                    let mut lock = app.images.lock().unwrap();
                    lock.push(image.clone());
                }
                filenames.push(image);
            }
        }
    }
    Ok(Json(filenames))
}

pub async fn get_first(
    Path(foreign_id): Path<u32>, 
    State(app): State<AppState>,
) -> impl IntoResponse {
    if let Ok(lock) = app.first_table.lock() {
        let mut results = lock.iter().filter_map(|data| {
            if data.foreign_id == foreign_id {
                Some(data.clone())
            } else {
                None
            }
        })
        .collect::<Vec<ImageData>>();

        // bahave like LEFT JOIN
        for result in results.iter_mut() {
            if let Ok(image) = app.images.lock() {
                if let Some(im) = image.iter().find(|im| *im == result) {
                    result.title = im.title.clone();
                }
            }
        }

        (StatusCode::OK, Json(results))
    } else {
        (StatusCode::NOT_FOUND, Json(Vec::new()))
    }
}

pub async fn get_second(
    Path(foreign_id): Path<u32>, 
    State(app): State<AppState>,
) -> impl IntoResponse {
    if let Ok(lock) = app.second_table.lock() {
        let mut results = lock.iter().filter_map(|data| {
            if data.foreign_id == foreign_id {
                Some(data.clone())
            } else {
                None
            }
        })
        .collect::<Vec<ImageData>>();

        // bahave like LEFT JOIN
        for result in results.iter_mut() {
            if let Ok(image) = app.images.lock() {
                if let Some(im) = image.iter().find(|im| *im == result) {
                    result.title = im.title.clone();
                }
            }
        }

        (StatusCode::OK, Json(results))
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
            payload.foreign_id = 1;
            lock.push(payload);
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
            payload.foreign_id = 1;
            lock.push(payload);
        }

        (StatusCode::OK, Json::<Vec<String>>(Vec::new()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
    }
}

pub async fn put_image(
    State(app): State<AppState>,
    Json(payload): Json<ImageData>,
) -> impl IntoResponse {
    if let Ok(mut lock) = app.images.lock() {
        if let Some(old) = lock.iter_mut().find(|data| **data == payload) {
            old.title = payload.title;
        }

        (StatusCode::OK, Json::<Vec<String>>(Vec::new()))
    } else {
        (StatusCode::NOT_FOUND, Json(Vec::new()))
    }
}

pub async fn delete_first(
    State(app): State<AppState>,
    Json(payloads): Json<Vec<u32>>,
) -> impl IntoResponse {
    if let Ok(mut lock) = app.first_table.lock() {
        for image_id in payloads {
            if let Some(pos) = lock.iter().position(|image| image.image_id == image_id) {
                lock.remove(pos);
            }
        }

        (StatusCode::OK, Json::<Vec<String>>(Vec::new()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
    }
}

pub async fn delete_second(
    State(app): State<AppState>,
    Json(payloads): Json<Vec<u32>>,
) -> impl IntoResponse {
    if let Ok(mut lock) = app.second_table.lock() {
        for image_id in payloads {
            if let Some(pos) = lock.iter().position(|image| image.image_id == image_id) {
                lock.remove(pos);
            }
        }

        (StatusCode::OK, Json::<Vec<String>>(Vec::new()))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
    }
}