// #[macro_use(concat_string)]
// extern crate concat_string;

mod abort;
mod binding;
mod image;
mod loader;

use dominator::{clone, Dom, events, get_id, html};
use futures_signals::{
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};

use js_sys::{Array, ArrayBuffer, Uint8Array};
use gloo_timers::callback::Timeout;
use std::rc::Rc;
use ulid::Ulid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, File, FileList, FormData, Headers, HtmlInputElement, RequestInit, Response, window};
use model::ImageData;

use abort::Abort;
use binding::{Viewer, ViewerOption};
use image::image_bytes_parser;
use loader::AsyncLoader;

struct App {
    loaded_first: Mutable<bool>,
    first_image_datas: MutableVec<ImageData>,
    images_redraw: Mutable<bool>,
}

impl App {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            loaded_first: Mutable::new(false),
            first_image_datas: MutableVec::new(),
            images_redraw: Mutable::new(false),
        })
    }
}

#[wasm_bindgen(start)]
pub fn main_js() {
    wasm_logger::init(wasm_logger::Config::default());

    // std::panic::set_hook(Box::new(on_panic));
    console_error_panic_hook::set_once();
    log::info!("wasm logging enabled");

    let app = App::new();
    dominator::append_dom(&get_id("app"), render(app));
}

fn render(app: Rc<App>) -> Dom {
    html!("div", {
        .future(app.loaded_first.signal().for_each(clone!(app => move |loaded| {
            clone!(app => async move {
                if !loaded {
                    let image_datas = get_first_images().await.unwrap();
                    if !image_datas.is_empty() {
                        {
                            let mut lock = app.first_image_datas.lock_mut();
                            lock.clear();
                            lock.extend(image_datas);
                        }
                        let elm = get_id("images-list");
                        Viewer::new(&elm).destroy();
                        app.images_redraw.set(true);
                    }
                    app.loaded_first.set(true);
                }
            })
        })))
        .future(app.images_redraw.signal_cloned().for_each(clone!(app => move |redraw| {
            if redraw {
                let elm = get_id("images-list");
                Timeout::new(100, clone!(app => move || {
                    let _ = Viewer::new_with_original(&elm, &ViewerOption::default().to_value());
                    app.images_redraw.set(false);
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
        .child(html!("br"))
        .child(html!("div", {
            .class("card")
            // default scroll-bar width edge=15, firefox=17
            // 2 cols : (1+128+1)*2 + scrollbar 20 = 280
            // 3 cols : (1+128+1)*3 + scrollbar 20 = 420
            // 4 cols : (1+128+1)*4 + scrollbar 20 = 540
            .style("max-width","280px")
            .child(html!("div", {
                .class("card-header")
                .children(&mut [
                    html!("label", {
                        .attr("for","file_upload")
                        .class(["btn","btn-outline-primary"])
                        .text("Add")
                    }),
                    html!("input", {
                        .attr("type","file")
                        .attr("id","file_upload")
                        .attr("accept","image/*")
                        .attr("capture","environment")
                        .attr("multiple","")
                        .class("d-none")
                        .event(clone!(app => move |e: events::Change| {
                            if let Some(input) = e.target() {
                                let file_input = input.dyn_into::<HtmlInputElement>().unwrap();
                                if let Some(files) = file_input.files() {
                                    if files.length() > 0 {
                                        let loader = AsyncLoader::new();
                                        loader.load(clone!(app, files => async move {
                                            let urls = post_files(&files).await.unwrap();
                                            let _ = post_first_images(&urls).await.unwrap();
    
                                            app.loaded_first.set(false);
    
                                            file_input.set_value("");
                                        }));
                                    }
                                }
                            }
                        }))
                    }),
                ])
            }))
            .child(html!("div", {
                .class(["card-body","p-0"])
                .child(html!("ul", {
                    .class(["d-flex","flex-wrap","m-0","p-0"])
                    .attr("id","images-list")
                    .style("overflow-y","auto")
                    // 2 cols : (1+128+1)*2 = 260
                    // 3 cols : (1+128+1)*3 = 390
                    // 4 cols : (1+128+1)*4 = 520
                    .style("height","260px")
                    .children_signal_vec(app.first_image_datas.signal_vec_cloned().map(|image_data| {
                        html!("li", {
                            .class("position-relative")
                            .style("margin","1px")
                            // .style("border","1px solid transparent")
                            // .style("float","left")
                            // .style("height","calc(100% / 3)")
                            // .style("margin","0 -1px -1px 0")
                            // .style("overflow","hidden")
                            // .style("width","calc(100% / 3 - 3px)")
                            .child(html!("img", {
                                .style("cursor","zoom-in")
                                .attr("data-original", &["images", &image_data.path].join("/"))
                                .attr("src", &["thumbs", &image_data.path].join("/"))
                                .attr("alt", &image_data.path)
                            }))
                            .child(html!("div", {
                                .class(["position-absolute","bottom-0","start-0","w-100","text-center","p-1"])
                                .style("font-size","8px")
                                .style("overflow","hidden")
                                .style("text-overflow","ellipsis")
                                .style("color","white")
                                .style("background-color","rgba(0,0,0,0.7)")
                                .style("z-index","9")
                                .style("pointer-events","none")
                                .text(&image_data.path)
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

async fn get_first_images() -> Result<Vec<ImageData>, String> {

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

async fn post_first_images(
    urls: &[String],
) -> Result<Vec<String>, String> {

    let image_datas = urls.iter().map(|url| {
        ImageData {
            image_id: 0,
            foreign_id: 1,
            path: url.to_string(),
            user: String::from("user"),
        }
    }).collect::<Vec<ImageData>>();

    let body_json = serde_json::to_string(&image_datas).map_err(|e| e.to_string())?;
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