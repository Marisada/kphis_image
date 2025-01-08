use futures::future::{abortable, AbortHandle};
use futures_signals::signal::Mutable;
use std::{
    future::Future,
    sync::atomic::{AtomicUsize, Ordering},
};
use wasm_bindgen_futures::spawn_local;

struct AsyncState {
    id: usize,
    handle: AbortHandle,
}

impl AsyncState {
    fn new(handle: AbortHandle) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);

        let id = ID.fetch_add(1, Ordering::SeqCst);

        Self { id, handle }
    }
}

pub struct AsyncLoader {
    loading: Mutable<Option<AsyncState>>,
}

impl Default for AsyncLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncLoader {
    pub fn new() -> Self {
        Self {
            loading: Mutable::new(None),
        }
    }

    // pub fn cancel(&self) {
    //     self.replace(None);
    // }

    fn replace(&self, value: Option<AsyncState>) {
        let mut loading = self.loading.lock_mut();

        if let Some(state) = loading.as_mut() {
            state.handle.abort();
        }

        *loading = value;
    }

    pub fn load<F>(&self, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let (fut, handle) = abortable(fut);

        let state = AsyncState::new(handle);
        let id = state.id;

        self.replace(Some(state));

        let loading = self.loading.clone();

        spawn_local(async move {
            if let Ok(()) = fut.await {
                let mut loading = loading.lock_mut();

                if let Some(current_id) = loading.as_ref().map(|x| x.id) {
                    // If it hasn't been overwritten with a new state...
                    if current_id == id {
                        *loading = None;
                    }
                }
            }
        });
    }

    // pub fn is_loading(&self) -> impl Signal<Item = bool> {
    //     self.loading.signal_ref(|x| x.is_some())
    // }
}
