use std::future::Future;

/// Executes a `async fn` and blocks on it.
/// Works for native and web.
pub fn block_on<F: Future<Output = ()> + 'static>(func: impl FnOnce() -> F) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(func());
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(func());
    }
}
