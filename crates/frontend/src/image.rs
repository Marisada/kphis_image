use dominator::{clone, Dom, events, html, with_node};
use futures_signals::{
    map_ref,
    signal::{Mutable, Signal, SignalExt},
    signal_vec::{MutableVec, SignalVecExt},
};
use gloo_timers::callback::Timeout;
use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};
use wasm_bindgen::prelude::*;
use web_sys::{HtmlButtonElement, HtmlInputElement};
use model::ImageData;

use crate::{
    App,
    binding::{Viewer, ViewerOption},
    fetch::{
        post_files, put_image,
        get_first_images, post_first_images, delete_first_images,
        get_second_images, post_second_images, delete_second_images,
    },
    mixins,
};

const ANIMATE_ROTATION_STYLE: &str = r#"
.sway {
  animation-name: swaying;
  animation-duration: 1s;
  animation-iteration-count: infinite
}
@keyframes swaying {
  0%   {rotate:-15deg}
  8%   {rotate:-10deg}
  16%  {rotate:-5deg}
  25%  {rotate: 0deg}
  34%  {rotate: 5deg}
  42%  {rotate: 10deg}
  50%  {rotate: 15deg}
  58%  {rotate: 10deg}
  66%  {rotate: 5deg}
  75%  {rotate: 0deg}
  84%  {rotate:-5deg}
  92%  {rotate:-10deg}
  100% {rotate:-15deg}
}"#;

#[derive(Clone)]
pub enum ImageUseAt {
    First,
    Second,
}

pub struct ImageCpn {
    id: usize,
    use_at: ImageUseAt,
    select_mode: Mutable<bool>,
    loaded: Mutable<bool>,

    viewer: Mutable<Option<Rc<Viewer>>>,

    image_datas: MutableVec<Rc<ImageData>>,
    images_redraw: Mutable<bool>,

    selected: Mutable<Vec<Rc<ImageData>>>,
    edited: Mutable<Option<ImageData>>,
    edit_title: Mutable<String>,
}

impl ImageCpn {
    pub fn new(use_at: ImageUseAt) -> Rc<Self> {
        static ID: AtomicUsize = AtomicUsize::new(1);
        let id = ID.fetch_add(1, Ordering::SeqCst);

        Rc::new(Self {
            id,
            use_at,
            select_mode: Mutable::new(false),
            loaded: Mutable::new(false),
            viewer: Mutable::new(None),
            image_datas: MutableVec::new(),
            images_redraw: Mutable::new(false),
            selected: Mutable::new(Vec::new()),
            edited: Mutable::new(None),
            edit_title: Mutable::new(String::new()),
        })
    }

    fn has_selected_signal(&self) -> impl Signal<Item = bool> {
        self.selected.signal_cloned().map(|v| !v.is_empty())
    }

    fn viewer_id(&self) -> String {
        ["images-list-", &self.id.to_string()].join("")
    }

    fn viewer_render(page: Rc<Self>, app: Rc<App>) {
        if let Some(viewer) = page.viewer.get_cloned() {
            viewer.update();
        } else {
            if let Some(elm) = app.get_id(&page.viewer_id()) {
                Timeout::new(100, clone!(page => move || {
                    let viewer = Viewer::new_with_original(&elm, &ViewerOption::default().to_value());
                    page.viewer.set(Some(Rc::new(viewer)));
                })).forget();
            }
        }
    }

    fn viewer_destroy(&self) {
        if let Some(viewer) = self.viewer.get_cloned() {
            viewer.destroy();
            self.viewer.set(None);
        }
    }

    async fn get_images(&self) {
        let image_datas = match self.use_at {
            ImageUseAt::First => get_first_images().await.unwrap(),
            ImageUseAt::Second => get_second_images().await.unwrap(),
        };
        if !image_datas.is_empty() {
            {
                let mut lock = self.image_datas.lock_mut();
                lock.clear();
                lock.extend(image_datas.into_iter().map(Rc::new));
            }
            self.viewer_destroy();
            self.images_redraw.set(true);
        }
    }

    async fn post_images(&self, images: &[Rc<ImageData>]) {
        match self.use_at {
            ImageUseAt::First => post_first_images(images).await.unwrap(),
            ImageUseAt::Second => post_second_images(images).await.unwrap(),
        };
    }

    async fn delete_images(&self, ids: &[u32]) {
        match self.use_at {
            ImageUseAt::First => delete_first_images(&ids).await.unwrap(),
            ImageUseAt::Second => delete_second_images(&ids).await.unwrap(),
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
                    app.loader.load(clone!(page => async move {
                        page.get_images().await;
                    }))
                }
                async {}
            })))
            .future(page.images_redraw.signal_cloned().for_each(clone!(app, page => move |redraw| {
                if redraw {
                    page.images_redraw.set(false);
                    Self::viewer_render(page.clone(), app.clone());
                }
                async {}
            })))
            .child(html!("style", { .text(ANIMATE_ROTATION_STYLE)}))
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
                            .class_signal("sway", app.loader.is_loading())
                        }))
                    }))
                    .child_signal(page.select_mode.signal().map(clone!(app, page => move |select_mode| {
                        (!select_mode).then(|| {
                            html!("div", {
                                .child(html!("label", {
                                    .class(["btn","btn-outline-primary","me-1"])
                                    .style_signal("opacity", app.loader.is_loading().map(|loading| {
                                        if loading {"0.7"} else {"1"}
                                    }))
                                    .text("เพิ่มรูป")
                                    .child(html!("input" => HtmlInputElement, {
                                        .attr("type","file")
                                        .attr("accept","image/*")
                                        .attr("capture","environment")
                                        .attr("multiple","")
                                        .class("d-none")
                                        .apply(mixins::other_true_signal_disable(app.loader.is_loading()))
                                        .event(clone!(app, page => move |e: events::Change| {
                                            if let Some(input) = e.target() {
                                                let file_input = input.dyn_into::<HtmlInputElement>().unwrap();
                                                if let Some(files) = file_input.files() {
                                                    if files.length() > 0 {
                                                        app.loader.load(clone!(page => async move {
                                                            let images = post_files(&files).await.unwrap()
                                                                .into_iter().map(Rc::new).collect::<Vec<Rc<ImageData>>>();
                                                            page.post_images(&images).await;
                                                            file_input.set_value("");
                                                            page.loaded.set(false);
                                                        }));
                                                    }
                                                }
                                            }
                                        }))
                                    }))
                                }))
                                .child_signal(page.image_datas.signal_vec_cloned().to_signal_cloned().map(clone!(app, page => move |datas| {
                                    (!datas.is_empty()).then(|| {
                                        html!("button" => HtmlButtonElement, {
                                            .attr("type","button")
                                            .class(["btn","btn-outline-primary","me-1"])
                                            .text("เลือก")
                                            .apply(mixins::other_true_signal_disable(app.loader.is_loading()))
                                            .event(clone!(page => move |_: events::Click| {
                                                page.viewer_destroy();
                                                page.select_mode.set(true);
                                            }))
                                        })
                                    })
                                })))
                                .child_signal(app.clipboard_images.signal_vec_cloned().to_signal_cloned().map(clone!(app, page => move |datas| {
                                    (!datas.is_empty()).then(|| {
                                        html!("button" => HtmlButtonElement, {
                                            .attr("type","button")
                                            .class(["btn","btn-outline-primary"])
                                            .text("วาง")
                                            .apply(mixins::other_true_signal_disable(app.loader.is_loading()))
                                            .event(clone!(app, page => move |_: events::Click| {
                                                // prevent duplicate image
                                                let recent_images = page.image_datas.lock_ref();
                                                let clipboard_images = app.clipboard_images.lock_ref();
                                                let mut selected = Vec::new();
                                                for image in clipboard_images.iter() {
                                                    if !recent_images.contains(image) {
                                                        selected.push(image.clone());
                                                    }
                                                }
                                                app.loader.load(clone!(page, selected => async move {
                                                    page.post_images(&selected).await;
                                                    // app.clipboard_images.lock_mut().clear();
                                                    page.loaded.set(false);
                                                }));
                                            }))
                                        })
                                    })
                                })))
                            })
                        })
                    })))
                    .child_signal(page.select_mode.signal().map(clone!(app, page => move |select_mode| {
                        select_mode.then(|| {
                            html!("div", {
                                .child_signal(page.has_selected_signal().map(clone!(app, page => move |has_selected| {
                                    (!has_selected).then(|| {
                                        html!("button", {
                                            .attr("type","button")
                                            .class(["btn","btn-outline-dark","me-1"])
                                            .text("ยกเลิก")
                                            .event(clone!(app, page => move |_: events::Click| {
                                                Self::viewer_render(page.clone(), app.clone());
                                                page.select_mode.set(false);
                                            }))
                                        })
                                    })
                                })))
                                .child_signal(page.has_selected_signal().map(clone!(app, page => move |has_selected| {
                                    (has_selected).then(|| {
                                        html!("div", {
                                            .child_signal(page.selected.signal_cloned().map(clone!(page => move |selected| {
                                                (selected.len() == 1).then(|| {
                                                    html!("button", {
                                                        .attr("type","button")
                                                        .class(["btn","btn-outline-success","me-1"])
                                                        .text("ข้อความ")
                                                        .event(clone!(page => move |_: events::Click| {
                                                            if let Some(outer) = selected.first() {
                                                                page.edited.set(Rc::into_inner(outer.clone()));
                                                            }
                                                        }))
                                                    })
                                                })
                                            })))
                                            .children(&mut [
                                                html!("button", {
                                                    .attr("type","button")
                                                    .class(["btn","btn-outline-primary","me-1"])
                                                    .text("สำเนา")
                                                    .event(clone!(app, page => move |_: events::Click| {
                                                        let mut selected_lock = page.selected.lock_mut();
                                                        if !selected_lock.is_empty() {
                                                            let mut clipboard_lock = app.clipboard_images.lock_mut();
                                                            clipboard_lock.clear();
                                                            clipboard_lock.extend(selected_lock.drain(0..));
                                                        }
                                                        Self::viewer_render(page.clone(), app.clone());
                                                        page.select_mode.set(false);
                                                    }))
                                                }),
                                                html!("button" => HtmlButtonElement, {
                                                    .attr("type","button")
                                                    .class(["btn","btn-outline-danger"])
                                                    .text("ลบ")
                                                    .apply(mixins::other_true_signal_disable(app.loader.is_loading()))
                                                    .event(clone!(app, page => move |_: events::Click| {
                                                        let ids;
                                                        {
                                                            let mut selected_lock = page.selected.lock_mut();
                                                            ids = selected_lock.iter().map(|image| image.image_id).collect::<Vec<u32>>();
                                                            selected_lock.clear();
                                                        }
                                                        if !ids.is_empty() {
                                                            app.loader.load(clone!(page, ids => async move {
                                                                page.delete_images(&ids).await;
                                                                page.get_images().await;
                                                            }));
                                                        }
                                                        Self::viewer_render(page.clone(), app.clone());
                                                        page.select_mode.set(false);
                                                    }))
                                                }),
                                            ])
                                        })
                                    })
                                })))
                            })
                        })
                    })))
                }))
                .child(html!("div", {
                    .class(["card-body","p-0"])
                    .child_signal(page.edited.signal_cloned().map(clone!(page => move |opt| {
                        opt.map(|image| {
                            html!("div", {
                                .class(["input-group","w-100"])
                                .children(&mut [
                                    html!("span", {
                                        .class("input-group-text")
                                        .text("ข้อความ")
                                    }),
                                    html!("input" => HtmlInputElement, {
                                        .class("form-control")
                                        .attr("value", &image.title.clone().unwrap_or_default())
                                        .with_node!(element => {
                                            .event(clone!(page => move |_:events::Input| {
                                                page.edit_title.set_neq(element.value());
                                            }))
                                        })
                                    }),
                                    html!("button", {
                                        .attr("type","button")
                                        .class(["btn","btn-primary"])
                                        .child(html!("i", {
                                            .class(["fas","fa-save"])
                                        }))
                                        .event(clone!(app, page => move |_:events::Click| {
                                            if let Some(mut edited) = page.edited.get_cloned() {
                                                let new_title = page.edit_title.get_cloned();
                                                edited.title = if new_title.is_empty() {
                                                    None
                                                } else {
                                                    Some(new_title)
                                                };
                                                app.loader.load(clone!(page => async move {
                                                    put_image(&edited).await.unwrap();
                                                    page.get_images().await;
                                                    page.edited.set(None);
                                                }));
                                            }
                                        }))
                                    }),
                                ])
                            })
                        })
                    })))
                    .child(html!("ul", {
                        .class(["d-flex","flex-wrap","m-0","p-0"])
                        .attr("id", &page.viewer_id())
                        .style("overflow-y","auto")
                        // 2 cols : (1+128+1)*2 = 260
                        // 3 cols : (1+128+1)*3 = 390
                        // 4 cols : (1+128+1)*4 = 520
                        .style("max-height","390px")
                        .children_signal_vec(page.image_datas.signal_vec_cloned().map(clone!(page => move |image_data| {
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
                                    .style_signal("cursor", page.select_mode.signal().map(|is_select| {
                                        if is_select {
                                            "pointer"
                                        } else {
                                            "zoom-in"
                                        }
                                    }))
                                    .attr("data-original", &["images", &image_data.path].join("/"))
                                    .attr("src", &["thumbs", &image_data.path].join("/"))
                                    .attr("alt", &image_data.path)
                                    .event(clone!(page, image_data => move |_:events::Click| {
                                        if page.select_mode.get() {
                                            let mut lock = page.selected.lock_mut();
                                            if let Some(pos) = lock.iter().position(|data| *data == image_data) {
                                                lock.remove(pos);
                                            } else {
                                                lock.push(image_data.clone());
                                            }
                                        }
                                    }))
                                }))
                                .child_signal(page.selected.signal_cloned().map(clone!(image_data => move |selected| {
                                    selected.contains(&image_data).then(|| {
                                        html!("i", {
                                            .class(["far","fa-circle-check","text-danger","position-absolute","end-0","top-0"])
                                            .style("font-size","30px")
                                            .style("pointer-events","none")
                                        })
                                    })
                                })))
                                .apply(|dom| {
                                    if let Some(title) = &image_data.title {
                                        dom.child(html!("div", {
                                            .class(["position-absolute","bottom-0","start-0","w-100","text-center","p-1"])
                                            .style("font-size","8px")
                                            .style("overflow","hidden")
                                            .style("text-overflow","ellipsis")
                                            .style("color","white")
                                            .style("background-color","rgba(0,0,0,0.7)")
                                            .style("z-index","9")
                                            .style("pointer-events","none")
                                            .text(title)
                                        }))
                                    } else {
                                        dom
                                    }
                                })
                            })
                        })))
                    }))
                }))
            }))
        })
    }
}
