use dominator::{clone, Dom, events, html};
use futures_signals::{
    map_ref,
    signal::{Mutable, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};
use gloo_timers::callback::Timeout;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{FileList, HtmlInputElement};
use model::ImageData;

use crate::{
    App,
    binding::{Viewer, ViewerOption},
    fetch::{
        post_files, 
        get_first_images, post_first_images,
        get_second_images, post_second_images,
    },
};

#[derive(Clone)]
pub enum ImageUseAt {
    First,
    Second,
}

pub struct ImageCpn {
    use_at: ImageUseAt,
    select_mode: Mutable<bool>,
    loaded: Mutable<bool>,
    image_datas: MutableVec<ImageData>,
    images_redraw: Mutable<bool>,
}

impl ImageCpn {
    pub fn new(use_at: ImageUseAt) -> Rc<Self> {
        Rc::new(Self {
            use_at,
            select_mode: Mutable::new(false),
            loaded: Mutable::new(false),
            image_datas: MutableVec::new(),
            images_redraw: Mutable::new(false),
        })
    }

    fn get_images(page: Rc<Self>, app: Rc<App>) {
        app.loader.load(clone!(app, page => async move {
            let image_datas = match page.use_at {
                ImageUseAt::First => get_first_images().await.unwrap(),
                ImageUseAt::Second => get_second_images().await.unwrap(),
            };
            if !image_datas.is_empty() {
                {
                    let mut lock = page.image_datas.lock_mut();
                    lock.clear();
                    lock.extend(image_datas);
                }
                if let Some(elm) = app.get_id("images-list") {
                    Viewer::new(&elm).destroy();
                    page.images_redraw.set(true);
                }
            }
        }))
    }

    fn post_images_from_files(files: FileList, page: Rc<Self>, app: Rc<App>) {
        app.loader.load(async move {
            let urls = post_files(&files).await.unwrap();
            page.post_images(&urls).await;
            page.loaded.set(false);
        });
    }

    fn post_images_from_clipboard(page: Rc<Self>, app: Rc<App>) {
        app.loader.load(clone!(app => async move {
            let urls = app.clipboard_images.lock_ref().iter().map(|data| data.path.clone()).collect::<Vec<String>>();
            page.post_images(&urls).await;
            app.clipboard_images.lock_mut().clear();
            page.loaded.set(false);
        }));
    }

    async fn post_images(&self, urls: &[String]) {
        match self.use_at {
            ImageUseAt::First => post_first_images(urls).await.unwrap(),
            ImageUseAt::Second => post_second_images(urls).await.unwrap(),
        };
    }

    pub fn render(page: Rc<Self>, app: Rc<App>) -> Dom {
        html!("div", {
            .future(map_ref! {
                let busy = app.loader.is_loading(),
                let loaded = page.loaded.signal() => 
                !busy && !loaded
            }.for_each(clone!(app, page => move |ready| {
                if ready {
                    page.loaded.set(true);
                    Self::get_images(page.clone(), app.clone());
                }
                async {}
            })))
            .future(page.images_redraw.signal_cloned().for_each(clone!(app, page => move |redraw| {
                if redraw {
                    if let Some(elm) = app.get_id("images-list") {
                        Timeout::new(100, clone!(page => move || {
                            let _ = Viewer::new_with_original(&elm, &ViewerOption::default().to_value());
                            page.images_redraw.set(false);
                        })).forget();
                    }
                }
                async {}
            })))
            .child(html!("div", {
                .class("card")
                // default scroll-bar width edge=15, firefox=17
                // 2 cols : (1+128+1)*2 + scrollbar 20 = 280
                // 3 cols : (1+128+1)*3 + scrollbar 20 = 420
                // 4 cols : (1+128+1)*4 + scrollbar 20 = 540
                .style("max-width","280px")
                .child(html!("div", {
                    .class(["card-header","p-2"])
                    .child(html!("div", {
                        .class(["position-absolute","end-0","top-0","overflow-hidden"])
                        .style("width","75px")
                        .style("height","55px")
                        .child(html!("i", {
                            .class(["far","fa-image","position-absolute","end-0","top-0"])
                            .style("font-size","70px")
                            .style("opacity","0.3")
                            .style("rotate","-15deg")
                        }))
                    }))
                    .child_signal(page.select_mode.signal().map(clone!(page => move |select_mode| {
                        (!select_mode).then(|| {
                            html!("div", {
                                .children(&mut [
                                    html!("label", {
                                        .attr("for","file_upload")
                                        .class(["btn","btn-outline-primary","me-1"])
                                        .text("Add")
                                    }),
                                    html!("input", {
                                        .attr("type","file")
                                        .attr("id","file_upload")
                                        .attr("accept","image/*")
                                        .attr("capture","environment")
                                        .attr("multiple","")
                                        .class("d-none")
                                        .event(clone!(app, page => move |e: events::Change| {
                                            if let Some(input) = e.target() {
                                                let file_input = input.dyn_into::<HtmlInputElement>().unwrap();
                                                if let Some(files) = file_input.files() {
                                                    if files.length() > 0 {
                                                        Self::post_images_from_files(files, page.clone(), app.clone());
                                                        file_input.set_value("");
                                                    }
                                                }
                                            }
                                        }))
                                    }),
                                ])
                                .child_signal(page.image_datas.signal_vec_cloned().to_signal_cloned().map(clone!(page => move |datas| {
                                    (!datas.is_empty()).then(|| {
                                        html!("button", {
                                            .attr("type","button")
                                            .class(["btn","btn-outline-primary"])
                                            .text("Select")
                                            .event(clone!(page => move |_: events::Click| {
                                                page.select_mode.set(true);
                                            }))
                                        })
                                    })
                                })))
                                .child_signal(app.clipboard_images.signal_vec_cloned().to_signal_cloned().map(clone!(app, page => move |datas| {
                                    (!datas.is_empty()).then(|| {
                                        html!("button", {
                                            .attr("type","button")
                                            .class(["btn","btn-outline-primary"])
                                            .text("Paste")
                                            .event(clone!(app, page => move |_: events::Click| {
                                                Self::post_images_from_clipboard(page.clone(), app.clone());
                                            }))
                                        })
                                    })
                                })))
                            })
                        })
                    })))
                    .child_signal(page.select_mode.signal().map(clone!(page => move |select_mode| {
                        select_mode.then(|| {
                            html!("div", {
                                .children(&mut [
                                    html!("button", {
                                        .attr("type","button")
                                        .class(["btn","btn-outline-primary"])
                                        .text("Delete")
                                        .event(clone!(page => move |_: events::Click| {
                                            // TODO
    
                                            page.select_mode.set(false);
                                        }))
                                    })
                                ])
                            })
                        })
                    })))
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
                        .style("max-height","390px")
                        .children_signal_vec(page.image_datas.signal_vec_cloned().map(|image_data| {
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
}
