// #[macro_use(concat_string)]
// extern crate concat_string;

mod abort;
mod binding;
mod loader;

use dominator::{clone, Dom, events, get_id, html};
use futures_signals::{
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};
use image::{imageops::FilterType, ImageReader, ImageFormat, ImageResult};
use js_sys::{Array, ArrayBuffer, Uint8Array};
use gloo_timers::callback::Timeout;
use std::io::Cursor;
use ulid::Ulid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, File, FileList, FormData, Headers, HtmlInputElement, RequestInit, Response, window};

use abort::Abort;
use binding::{Viewer, ViewerOption};
use loader::AsyncLoader;

const MAX_WIDTH: u32 = 720; // 9x64=576, 9x80=720, 9x96=864
const MAX_HEIGHT: u32 = 1280; // 16x64=1024, 16x80=1280, 16*96=1536
const THUMB_WIDTH: u32 = 144; // 9x16
const THUMB_HEIGHT: u32 = 256; // 16x16

#[wasm_bindgen(start)]
pub fn main_js() {
    wasm_logger::init(wasm_logger::Config::default());

    // std::panic::set_hook(Box::new(on_panic));
    console_error_panic_hook::set_once();
    log::info!("wasm logging enabled");

    dominator::append_dom(&dominator::get_id("app"), render());
}

fn render() -> Dom {

    let thumb_paths: MutableVec<String> = MutableVec::new();
    let images_redraw = Mutable::new(false);

    html!("div", {
        .future(images_redraw.signal_cloned().for_each(clone!(images_redraw => move |redraw| {
            if redraw {
                let elm = get_id("images-list");
                Timeout::new(100, clone!(images_redraw => move || {
                    let _ = Viewer::new_with_original(&elm, &ViewerOption::default().to_value());
                    images_redraw.set(false);
                })).forget();
            }
            async {}
        })))
        .text("Hello World !!!")
        .child(html!("br"))
        .child(html!("a", {
            .attr("href","api/greet")
            .attr("target","_blank")
            .text("Greeting page")
        }))
        .child(html!("div", {
            .children(&mut [
                html!("label", {
                    .attr("for","file_upload")
                    .style("padding","5px 10px")
                    .style("border-radius","5px")
                    .style("border","1px ridge black")
                    .text("กรุณาเลือกไฟล์")
                }),
                html!("input", {
                    .attr("type","file")
                    .attr("id","file_upload")
                    // .attr("accept",".png,.jpg,.jpeg,.webp")
                    .attr("accept","image/*")
                    .attr("capture","environment")
                    .attr("multiple","")
                    .style("opacity","0")
                    .event(clone!(thumb_paths, images_redraw => move |e: events::Change| {
                        if let Some(input) = e.target() {
                            let file_input = input.dyn_into::<HtmlInputElement>().unwrap();
                            if let Some(files) = file_input.files() {
                                if files.length() > 0 {
                                    let loader = AsyncLoader::new();
                                    loader.load(clone!(thumb_paths, images_redraw, files => async move {
                                        let urls = post_files(&files).await.unwrap();
                                        let mut lock = thumb_paths.lock_mut();
                                        lock.clear();
                                        lock.extend(urls);

                                        let elm = get_id("images-list");
                                        Viewer::new(&elm).destroy();
                                        images_redraw.set(true);
                                    }))
                                }
                            }
                        }
                    }))
                }),
            ])
            .child(html!("div", {
                .attr("id", "images-container")
                .child(html!("ul", {
                    .class("clearfix")
                    .attr("id","images-list")
                    .style("list-style","none")
                    .style("margin","0")
                    .style("padding","0")
                    .style("max-width","400px")
                    .children_signal_vec(thumb_paths.signal_vec_cloned().map(|url| {
                        html!("li", {
                            .style("border","1px solid transparent")
                            .style("float","left")
                            .style("height","calc(100% / 3)")
                            .style("margin","0 -1px -1px 0")
                            .style("overflow","hidden")
                            .style("width","calc(100% / 3 - 3px)")
                            .child(html!("img", {
                                .style("cursor","-webkit-zoom-in")
                                .style("cursor","zoom-in")
                                .style("width","100%")
                                .attr("data-original", &["images",&url].join("/"))
                                .attr("src", &["thumbs",&url].join("/"))
                                .attr("alt",&url)
                            }))
                        })
                    }))
                }))
            }))
        }))
    })
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
    img_u8a.copy_from(&bytes);
    // Uint8Array to Array
    let img_array = Array::new_with_length(1);
    img_array.set(0, img_u8a.into());
    // Array to Blob
    Blob::new_with_u8_array_sequence(&img_array)
}

fn image_bytes_parser(raw_data: &[u8]) -> ImageResult<(Vec<u8>, Vec<u8>)> {
    let raw_image = ImageReader::new(Cursor::new(raw_data)).with_guessed_format()?.decode()?;
    let raw_w = raw_image.width();
    let image = if raw_w > MAX_WIDTH {
        // see filters detail at https://docs.rs/image/latest/image/imageops/enum.FilterType.html
        raw_image.resize(MAX_WIDTH, MAX_HEIGHT, FilterType::Triangle)
    } else {
        raw_image
    };
    let mut res_image = Vec::new();
    image.write_to(&mut Cursor::new(&mut res_image), ImageFormat::WebP)?;

    let thumb = image.thumbnail(THUMB_WIDTH, THUMB_HEIGHT);
    let mut res_thumb = Vec::new();
    thumb.write_to(&mut Cursor::new(&mut res_thumb), ImageFormat::WebP)?;

    Ok((res_image, res_thumb))
}

async fn post_files(
    filelist: &FileList,
) -> Result<Vec<String>, String> {
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
    match post_multipart("/api/upload", &form_data).await {
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