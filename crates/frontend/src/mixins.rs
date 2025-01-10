use dominator::{with_node, DomBuilder};
use futures_signals::signal::{Signal, SignalExt};
use web_sys::{HtmlButtonElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};

/// this input element will disabled when other signal is true
pub fn other_true_signal_disable<T, S>(other: S) -> impl FnOnce(DomBuilder<T>) -> DomBuilder<T>
where
    T: CanDisable + std::clone::Clone + 'static,
    S: Signal<Item = bool> + 'static,
{
    #[inline]
    move |dom| {
        with_node!(dom, element => {
            .future(other.for_each(move |v| {
                element.set_disabled(v);
                async {}
            }))
        })
    }
}


pub trait CanDisable {
    fn set_disabled(&self, value: bool);
}

impl CanDisable for HtmlButtonElement {
    #[inline]
    fn set_disabled(&self, value: bool) {
        self.set_disabled(value)
    }
}
impl CanDisable for HtmlInputElement {
    #[inline]
    fn set_disabled(&self, value: bool) {
        self.set_disabled(value)
    }
}
impl CanDisable for HtmlSelectElement {
    #[inline]
    fn set_disabled(&self, value: bool) {
        self.set_disabled(value)
    }
}
impl CanDisable for HtmlTextAreaElement {
    #[inline]
    fn set_disabled(&self, value: bool) {
        self.set_disabled(value)
    }
}