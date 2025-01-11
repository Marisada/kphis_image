// #[macro_use(concat_string)]
// extern crate concat_string;

mod abort;
mod binding;
mod fetch;
mod image;
mod image_parser;
mod mixins;
mod loader;

use dominator::{Dom, html};
use futures_signals::signal_vec::MutableVec;
use std::{
    rc::Rc,
    thread::LocalKey,
};
use wasm_bindgen::prelude::*;
use web_sys::{Element, Window, window};

use model::ImageData;

use image::{ImageCpn, ImageOf};
use loader::AsyncLoader;

thread_local! {
    static WINDOW: Window = window().unwrap();
}

struct App {
    window: &'static LocalKey<Window>,
    loader: AsyncLoader,
    pub clipboard_images: MutableVec<Rc<ImageData>>,
}

impl App {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            window: &WINDOW,
            loader: AsyncLoader::default(),
            clipboard_images: MutableVec::new(),
        })
    }

    fn render(app: Rc<Self>) -> Dom {
        html!("div", {
            .text("Hello World !!!")
            .child(html!("br"))
            .child(html!("a", {
                .attr("href","api/greet")
                .attr("target","_blank")
                .text("Greeting page")
            }))
            .child(html!("br"))
            .child(html!("div", {
                .class(["row","m-0"])
                .children(&mut [
                    html!("div", {
                        .style("width","100px")
                    }),
                    html!("div", {
                        .class(["mt-3","p-0"])
                        .style("max-width","391px")
                        .style("max-height","400px")
                        .style("border","1px solid red")
                        .child(ImageCpn::render("50vh", ImageCpn::new(ImageOf::First, false), app.clone()))  
                    }),
                    html!("div", {
                        .style("width","100px")
                    }),
                    html!("div", {
                        .class(["mt-3","p-0"])
                        .style("max-width","392px")
                        .style("max-height","500px")
                        .style("border","1px solid red")
                        .child(ImageCpn::render("300px", ImageCpn::new(ImageOf::Second, true), app)) 
                    }),
                ])
            }))
        })
    }
    pub fn get_id(&self, id: &str) -> Option<Element> {
        self.window
            .with(|w| w.document().and_then(|d| d.get_element_by_id(id)))
    }
}

#[wasm_bindgen(start)]
pub fn main_js() {
    wasm_logger::init(wasm_logger::Config::default());

    // std::panic::set_hook(Box::new(on_panic));
    console_error_panic_hook::set_once();
    log::info!("wasm logging enabled");

    let app = App::new();
    if let Some(elm) = app.get_id("app") {
        dominator::append_dom(&elm, App::render(app));
    }
}

#[inline]
pub fn str_some(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}