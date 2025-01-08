use axum::{
    body::Body,
    extract::Multipart,
    http::StatusCode,
    response::{Html, Response}, Json,
};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::info;

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
