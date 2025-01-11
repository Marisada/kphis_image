use dominator::{clone, Dom, events, html, with_node};
use futures_signals::{
    map_ref,
    signal::{not, Mutable, Signal, SignalExt},
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
    mixins, str_some,
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
pub enum ImageOf {
    First,
    Second,
}

pub struct ImageCpn {
    is_dark: bool,

    id: usize,
    use_at: ImageOf,
    select_mode: Mutable<bool>,
    loaded: Mutable<bool>,

    viewer: Mutable<Option<Rc<Viewer>>>,

    image_datas: MutableVec<Rc<ImageData>>,
    images_redraw: Mutable<bool>,

    selected: Mutable<Vec<Rc<ImageData>>>,
    edited: Mutable<Option<ImageData>>,
    old_title: Mutable<String>,
    edited_title: Mutable<String>,
}

impl ImageCpn {
    pub fn new(use_at: ImageOf, is_dark: bool) -> Rc<Self> {
        static ID: AtomicUsize = AtomicUsize::new(1);
        let id = ID.fetch_add(1, Ordering::SeqCst);

        Rc::new(Self {
            is_dark,

            id,
            use_at,
            select_mode: Mutable::new(false),
            loaded: Mutable::new(false),
            viewer: Mutable::new(None),
            image_datas: MutableVec::new(),
            images_redraw: Mutable::new(false),
            selected: Mutable::new(Vec::new()),
            edited: Mutable::new(None),
            old_title: Mutable::new(String::new()),
            edited_title: Mutable::new(String::new()),
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
        } else if let Some(elm) = app.get_id(&page.viewer_id()) {
            Timeout::new(100, clone!(page => move || {
                let viewer = Viewer::new_with_original(&elm, &ViewerOption::default().to_value());
                page.viewer.set(Some(Rc::new(viewer)));
            })).forget();
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
            ImageOf::First => get_first_images().await.unwrap(),
            ImageOf::Second => get_second_images().await.unwrap(),
        };
        let mut lock = self.image_datas.lock_mut();
        lock.clear();
        lock.extend(image_datas.into_iter().map(Rc::new));
    }

    async fn post_images(&self, images: &[Rc<ImageData>]) {
        match self.use_at {
            ImageOf::First => post_first_images(images).await.unwrap(),
            ImageOf::Second => post_second_images(images).await.unwrap(),
        };
    }

    async fn delete_images(&self, ids: &[u32]) {
        match self.use_at {
            ImageOf::First => delete_first_images(ids).await.unwrap(),
            ImageOf::Second => delete_second_images(ids).await.unwrap(),
        };
    }

    fn edit_title(page: Rc<Self>, app: Rc<App>) {
        if let Some(mut edited) = page.edited.get_cloned() {
            edited.title = str_some(page.edited_title.get_cloned());
            app.loader.load(clone!(page => async move {
                put_image(&edited).await.unwrap();
                page.get_images().await;
            }));
        }
    }

    /// header height=48px, title input height=36px, the rest is images_max_height
    pub fn render(images_max_height: &'static str, page: Rc<Self>, app: Rc<App>) -> Dom {

        html!("div", {
            .attr("data-bs-theme",if page.is_dark {"dark"} else {"light"})
            .future(map_ref! {
                let busy = app.loader.is_loading(),
                let loaded = page.loaded.signal() => 
                !busy && !loaded
            }.for_each(clone!(app, page => move |ready| {
                if ready {
                    page.loaded.set(true);
                    app.loader.load(clone!(page => async move {
                        page.get_images().await;
                        page.viewer_destroy();
                        page.images_redraw.set(true);
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
                .children(&mut [
                    html!("div", {
                        .class(["card-header","border-bottom-0","p-2"])
                        .child(html!("div", {
                            .class(["position-absolute","end-0","top-0","overflow-hidden"])
                            .style("width","75px")
                            // not btn-sm
                            // .style("height","55px")
                            // btn-sm
                            .style("height","47px")
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
                                        .class(["btn","btn-sm","btn-primary","me-1"])
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
                                                .class(["btn","btn-sm","btn-primary","me-1"])
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
                                                .class(["btn","btn-sm","btn-primary"])
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
                                        (has_selected).then(|| {
                                            html!("span", {
                                                .children(&mut [
                                                    html!("button", {
                                                        .attr("type","button")
                                                        .class(["btn","btn-sm","btn-primary","me-1"])
                                                        .text("สำเนา")
                                                        .event(clone!(app, page => move |_: events::Click| {
                                                            page.select_mode.set(false);
                                                            let mut selected_lock = page.selected.lock_mut();
                                                            if !selected_lock.is_empty() {
                                                                let mut clipboard_lock = app.clipboard_images.lock_mut();
                                                                clipboard_lock.clear();
                                                                clipboard_lock.extend(selected_lock.drain(0..));
                                                            }
                                                            Self::viewer_render(page.clone(), app.clone());
                                                        }))
                                                    }),
                                                    html!("button" => HtmlButtonElement, {
                                                        .attr("type","button")
                                                        .class(["btn","btn-sm","btn-danger","me-1"])
                                                        .text("ลบ")
                                                        .apply(mixins::other_true_signal_disable(app.loader.is_loading()))
                                                        .event(clone!(app, page => move |_: events::Click| {
                                                            page.edited.set_neq(None);
                                                            page.select_mode.set(false);
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
                                                                    page.viewer_destroy();
                                                                    page.images_redraw.set(true);
                                                                }));
                                                            } else {
                                                                Self::viewer_render(page.clone(), app.clone());
                                                            }
                                                        }))
                                                    }),
                                                ])
                                            })
                                        })
                                    })))
                                    .child_signal(page.has_selected_signal().map(clone!(page => move |has_selected| {
                                        (!has_selected).then(|| {
                                            html!("button", {
                                                .attr("type","button")
                                                .class(["btn","btn-sm","btn-primary","me-1"])
                                                .text("ทั้งหมด")
                                                .event(clone!(page => move |_: events::Click| {
                                                    let mut lock = page.selected.lock_mut();
                                                    lock.clear();
                                                    lock.extend(page.image_datas.lock_ref().to_vec());
                                                    if lock.len() == 1 {
                                                        page.edited.set(lock.first().map(ImageData::from_rc_ref));
                                                    } else {
                                                        page.edited.set_neq(None);
                                                    }
                                                }))
                                            })
                                        })
                                    })))
                                    .child(html!("button", {
                                        .attr("type","button")
                                        .class(["btn","btn-sm","btn-secondary","me-1"])
                                        .text("ยกเลิก")
                                        .event(clone!(app, page => move |_: events::Click| {
                                            page.edited.set_neq(None);
                                            page.select_mode.set(false);
                                            page.selected.lock_mut().clear();
                                            Self::viewer_render(page.clone(), app.clone());
                                        }))
                                    }))
                                })
                            })
                        })))
                    }),
                    html!("div", {
                        .class(["card-body","p-0"])
                        .child_signal(page.edited.signal_cloned().map(clone!(app, page => move |opt| {
                            opt.map(|image| {
                                html!("div", {
                                    .class(["input-group","w-100"])
                                    .children(&mut [
                                        html!("input" => HtmlInputElement, {
                                            .class(["form-control","form-control-sm","rounded-0"])
                                            .attr("value", &image.title.clone().unwrap_or_default())
                                            .with_node!(element => {
                                                .event(clone!(app, page => move |e: events::KeyUp| {
                                                    if (e.key() == "Enter") && (page.edited_title.get_cloned() != page.old_title.get_cloned()) {
                                                        Self::edit_title(page.clone(), app.clone());
                                                        page.edited.set(None);
                                                        page.selected.lock_mut().clear();
                                                    } else {
                                                        page.edited_title.set_neq(element.value());
                                                    }
                                                }))
                                            })
                                        }),
                                        html!("button" => HtmlButtonElement, {
                                            .attr("type","button")
                                            .class(["btn","btn-sm","btn-primary"])
                                            .child(html!("i", {
                                                .class(["fas","fa-check"])
                                            }))
                                            .apply(mixins::other_true_signal_disable(map_ref!{
                                                let old = page.old_title.signal_cloned(),
                                                let changed = page.edited_title.signal_cloned() =>
                                                old == changed
                                            }))
                                            .event(clone!(app, page => move |_:events::Click| {
                                                Self::edit_title(page.clone(), app.clone());
                                                page.edited.set(None);
                                                page.selected.lock_mut().clear();
                                            }))
                                        }),
                                        html!("button" => HtmlButtonElement, {
                                            .attr("type","button")
                                            .class(["btn","btn-sm","btn-danger","border-0","rounded-0"])
                                            .child(html!("i", {
                                                .class(["fas","fa-x"])
                                            }))
                                            .apply(mixins::other_true_signal_disable(map_ref!{
                                                let old = page.old_title.signal_cloned(),
                                                let changed = page.edited_title.signal_cloned() =>
                                                old.is_empty() || changed.is_empty()
                                            }))
                                            .event(clone!(app, page => move |_:events::Click| {
                                                page.edited_title.set(String::new());
                                                Self::edit_title(page.clone(), app.clone());
                                                page.edited.set(None);
                                                page.selected.lock_mut().clear();
                                            }))
                                        }),
                                    ])
                                })
                            })
                        })))
                        .child(html!("div", {
                            .child(html!("div", {
                                .style("overflow-y","auto")
                                .style("max-height", images_max_height)
                                .child(html!("ul", {
                                    .class(["d-flex","flex-wrap","bg-secondary","m-0","p-0"])
                                    .attr("id", &page.viewer_id())
                                    .style("list-style-type","none")
                                    .children_signal_vec(page.image_datas.signal_vec_cloned().map(clone!(page => move |image_data| {
                                        html!("li", {
                                            .class("position-relative")
                                            .style("margin","1px")
                                            .style("flex-grow","1")
                                            .child(html!("img", {
                                                .class("w-100")
                                                .style_signal("cursor", page.select_mode.signal().map(|is_select| {
                                                    if is_select {
                                                        "pointer"
                                                    } else {
                                                        "zoom-in"
                                                    }
                                                }))
                                                .attr("data-original", &["images", &image_data.path].join("/"))
                                                .attr("src", &["thumbs", &image_data.path].join("/"))
                                                .attr("alt", &image_data.title.clone().unwrap_or(String::from("ไม่มีคำบรรยาย")))
                                                .event(clone!(page, image_data => move |_:events::Click| {
                                                    if page.select_mode.get() {
                                                        let mut lock = page.selected.lock_mut();
                                                        if let Some(pos) = lock.iter().position(|data| *data == image_data) {
                                                            lock.remove(pos);
                                                        } else {
                                                            lock.push(image_data.clone());
                                                        }
                                                        if lock.len() == 1 {
                                                            page.edited.set(lock.first().map(ImageData::from_rc_ref));
                                                            // always this image_data
                                                            let title = image_data.title.clone().unwrap_or_default();
                                                            page.old_title.set_neq(title.clone());
                                                            page.edited_title.set_neq(title);
                                                        } else {
                                                            page.edited.set_neq(None);
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
                                                        .class(["position-absolute","bottom-0","start-0","w-100","text-white","text-center","overflow-hidden","p-1"])
                                                        .style("font-size","8px")
                                                        .style("text-overflow","ellipsis")
                                                        .style("background-color","rgba(0,0,0,0.7)")
                                                        .style("pointer-events","none")
                                                        .style("z-index","9")
                                                        .text(title)
                                                    }))
                                                } else {
                                                    dom
                                                }
                                            })
                                        })
                                    })))
                                    .children_signal_vec(page.image_datas.signal_vec_cloned().len().map(clone!(app, page => move |data_len| {
                                        if let Some(elm) = app.get_id(&page.viewer_id()) {
                                            let cols = (elm.client_width() / 130) as usize;
                                            let remains = data_len % cols;
                                            let mut doms = Vec::new();
                                            for _ in 0..(cols - remains) {
                                                doms.push(blank_image());
                                            }
                                            doms
                                        } else {
                                            Vec::new()
                                        }
                                    })).to_signal_vec())
                                }))
                            }))
                        }))
                    }),
                    html!("div", {
                        .class(["card-footer","border-top-0"])
                        .class_signal("p-0", page.image_datas.signal_vec_cloned().is_empty())
                        .class_signal("p-1", not(page.image_datas.signal_vec_cloned().is_empty()))
                    }),
                ])
            }))
        })
    }
}

fn blank_image() -> Dom {
    html!("li", {
        .style("flex-grow","1")
        .child(html!("div", {
            .class("w-100")
            // thumb size 128 + margin 1 both end
            .style("min-width","130px")
        }))
    })
}
