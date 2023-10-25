use cfg_if::cfg_if;
use std::future::Future;

pub fn spawn_local<F>(fut: F)
where
    F: Future<Output = ()> + 'static,
{
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            wasm_bindgen_futures::spawn_local(fut)
        } else if #[cfg(any(test, doctest, feature = "tokio"))] {
            tokio::task::spawn_local(fut);
        } else {
            futures::executor::block_on(fut)
        }
    }
}

pub fn spawn<F>(fut: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            wasm_bindgen_futures::spawn_local(fut)
        } else if #[cfg(any(test, doctest, feature = "tokio"))] {
            tokio::task::spawn(fut);
        }  else {
            futures::executor::block_on(fut)
        }
    }
}