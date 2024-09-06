use std::future::Future;

/// Executes a `async fn` and blocks on it.
/// Works for native and web.
pub fn block_on<T, F: Future<Output = T> + 'static>(fut: F) -> T
where
    T: 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(fut)
    }
    #[cfg(target_arch = "wasm32")]
    {
        let outer_t = std::rc::Rc::new(std::cell::Cell::new(Option::<T>::None));
        let t = outer_t.clone();

        wasm_bindgen_futures::spawn_local(async move { t.set(Some(fut.await)) });

        let ret = std::rc::Rc::try_unwrap(outer_t);
        match ret {
            Ok(ret) => ret.into_inner().unwrap(),
            Err(_) => {
                panic!("Failed to unwrap return value");
            }
        }
    }
}
