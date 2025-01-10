use js_sys::{Array, ArrayBuffer, Uint8Array};
use ulid::Ulid;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, File, FileList, FormData, Headers, RequestInit, Response, window};
use model::ImageData;

use crate::{
    abort::Abort,
    image_parser::image_bytes_parser,
};

pub async fn get_first_images() -> Result<Vec<ImageData>, String> {

    match fetch_json_api("/api/first/1", "GET", None).await {
        Ok((response, true)) => {
            let response: Vec<ImageData> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

pub async fn get_second_images() -> Result<Vec<ImageData>, String> {

    match fetch_json_api("/api/second/1", "GET", None).await {
        Ok((response, true)) => {
            let response: Vec<ImageData> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

pub async fn post_first_images(
    images: &[Rc<ImageData>],
) -> Result<Vec<String>, String> {

    let body_json = serde_json::to_string(images).map_err(|e| e.to_string())?;
    let body = serde_wasm_bindgen::to_value(&body_json).map_err(|e| e.to_string())?;

    match fetch_json_api("/api/first", "POST", Some(&body)).await {
        Ok((response, true)) => {
            let response: Vec<String> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

pub async fn post_second_images(
    images: &[Rc<ImageData>],
) -> Result<Vec<String>, String> {

    let body_json = serde_json::to_string(&images).map_err(|e| e.to_string())?;
    let body = serde_wasm_bindgen::to_value(&body_json).map_err(|e| e.to_string())?;

    match fetch_json_api("/api/second", "POST", Some(&body)).await {
        Ok((response, true)) => {
            let response: Vec<String> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

pub async fn put_image(
    image: &ImageData,
) -> Result<Vec<String>, String> {

    let body_json = serde_json::to_string(image).map_err(|e| e.to_string())?;
    let body = serde_wasm_bindgen::to_value(&body_json).map_err(|e| e.to_string())?;

    match fetch_json_api("/api/image", "PUT", Some(&body)).await {
        Ok((response, true)) => {
            let response: Vec<String> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

pub async fn delete_first_images(ids: &[u32]) -> Result<Vec<String>, String> {

    let body_json = serde_json::to_string(&ids).map_err(|e| e.to_string())?;
    let body = serde_wasm_bindgen::to_value(&body_json).map_err(|e| e.to_string())?;

    match fetch_json_api("/api/first", "DELETE", Some(&body)).await {
        Ok((response, true)) => {
            let response: Vec<String> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

pub async fn delete_second_images(ids: &[u32]) -> Result<Vec<String>, String> {

    let body_json = serde_json::to_string(&ids).map_err(|e| e.to_string())?;
    let body = serde_wasm_bindgen::to_value(&body_json).map_err(|e| e.to_string())?;

    match fetch_json_api("/api/second", "DELETE", Some(&body)).await {
        Ok((response, true)) => {
            let response: Vec<String> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

pub async fn fetch_json_api(
    url: &str,
    method: &str,
    body: Option<&JsValue>,
) -> Result<(JsValue, bool), JsValue> {
    let abort = Abort::new()?;

    let headers = Headers::new()?;
    headers.set("Accept", "application/json")?;
    headers.set("Content-Type", "application/json")?;

    let w = window().unwrap();
    let init = RequestInit::new();
    init.set_method(method);
    init.set_headers(&headers);
    if let Some(b) = body {
        init.set_body(b);
    }
    init.set_signal(Some(&abort.signal()));
    let future = w.fetch_with_str_and_init(url, &init);

    let response = JsFuture::from(future).await?.unchecked_into::<Response>();

    let value = JsFuture::from(response.json()?).await?;

    if response.ok() {
        Ok((value, true))
    } else {
        Ok((value, false))
    }
}

pub async fn post_files(
    filelist: &FileList,
) -> Result<Vec<ImageData>, String> {
    let form_data = FormData::new().unwrap();
    for i in 0..filelist.length() {
        if let Some(file) = filelist.item(i) {
            let file_buf = file_to_bytes(&file).await.unwrap();
            // Parse image
            let (image, thumb) = image_bytes_parser(&file_buf).unwrap();

            let image_blob = bytes_to_blob(&image).await.unwrap();
            let thumb_blob = bytes_to_blob(&thumb).await.unwrap();
            let path_with_filename = new_ulid_to_path();
            form_data.append_with_blob_and_filename("images", &image_blob, &path_with_filename).unwrap();
            form_data.append_with_blob_and_filename("thumbs", &thumb_blob, &path_with_filename).unwrap();
        }
    }
    match post_multipart("/api/image", &form_data).await {
        Ok((response, true)) => {
            let response: Vec<ImageData> = serde_wasm_bindgen::from_value(response)
                .map_err(|e| e.to_string())?;
            Ok(response)
        }
        Ok((app_error, false)) => {
            let error: String = serde_wasm_bindgen::from_value(app_error)
                .map_err(|e| e.to_string())?;
            Err(error)
        }
        Err(e) => {
            Err(e.as_string().unwrap_or(String::from("fetch error")))
        }
    }
}

async fn post_multipart(
    url: &str,
    body: &FormData,
) -> Result<(JsValue, bool), JsValue> {
    let abort = Abort::new()?;

    let headers = Headers::new()?;
    headers.set("Accept", "multipart/form-data")?;
    // if let Some(bearer) = app.token() {
    //     headers.set("Authorization", &concat_string!("Bearer ", bearer))?;
    // }

    let w = window().unwrap();
    let init = RequestInit::new();
    init.set_method("POST");
    init.set_headers(&headers);
    init.set_body(body);
    init.set_signal(Some(&abort.signal()));
    let future = w.fetch_with_str_and_init(url, &init);

    let response = JsFuture::from(future).await?.unchecked_into::<Response>();

    // if response.status() == 401 {
    //     log::debug!("401 from server, remove user and redirect to index page");
    //     app.remove_user_and_go_index();
    // }

    if response.ok() {
        let value = JsFuture::from(response.json()?).await?;
        Ok((value, true))
    } else {
        let value = JsFuture::from(response.text()?).await?;
        Ok((value, false))
    }
}

async fn file_to_bytes(file: &File) -> Result<Vec<u8>, JsValue> {
    // File blob to ArrayBuffer
    let file_arr = JsFuture::from(file.array_buffer()).await?.unchecked_into::<ArrayBuffer>();
    let file_u8a = Uint8Array::new(&file_arr);
    // Unit8Array to [u8]
    let mut file_buf = vec![0; file_u8a.length() as usize];
    file_u8a.copy_to(&mut file_buf);
    Ok(file_buf)
}

async fn bytes_to_blob(bytes: &[u8]) -> Result<Blob, JsValue> {
    // [u8] to Uint8Array
    let img_u8a = Uint8Array::new_with_length(bytes.len() as u32);
    img_u8a.copy_from(bytes);
    // Uint8Array to Array
    let img_array = Array::new_with_length(1);
    img_array.set(0, img_u8a.into());
    // Array to Blob
    Blob::new_with_u8_array_sequence(&img_array)
}

/// new Ulid to `01J/G0/M004KYHATX7J2W7MB28X4.webp`
fn new_ulid_to_path() -> String {
    let mut s = Ulid::new().to_string();
    s.insert_str(s.len(), ".webp");
    s.insert(5, '/');
    s.insert(3, '/');
    s
}

#[cfg(test)]
pub mod tests {

    #[test]
    pub fn test_new_ulid_to_path() {
        let mut s = String::from("01JG0M004KYHATX7J2W7MB28X4");
        s.insert_str(s.len(), ".webp");
        s.insert(5, '/');
        s.insert(3, '/');
        assert_eq!(s, String::from("01J/G0/M004KYHATX7J2W7MB28X4.webp"));
    }
}