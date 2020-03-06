use crate::req::Request;
use lazycell::AtomicLazyCell;
use std::sync::Arc;

#[derive(Clone)]
pub struct LazyReqStore {
    store: Arc<AtomicLazyCell<Request>>,
}

impl LazyReqStore {
    pub fn empty() -> LazyReqStore {
        LazyReqStore {
            store: Arc::new(AtomicLazyCell::NONE),
        }
    }

    pub fn filled(&self) -> bool {
        self.store.filled()
    }

    pub fn fill(&self, req: Request) -> Result<(), Request> {
        self.store.fill(req)
    }

    pub fn borrow(&self) -> Option<&Request> {
        self.store.borrow()
    }
}

pub mod filters {
    /// Create a `Filter` that extracts `LazyReqStore`.
    pub fn store() -> impl Filter<Extract = (LazyReqStore,), Error = Rejection> {
        filters::ext::optional::<LazyReqStore>().and_then(|opt| {
            match opt {
                Some(store) => future::ok(store),
                None => {
                    log::error!("Your Filter has to be wrapped by `JsonRpcService`");
                    // TODO return custom error
                    future::err(reject::reject())
                }
            };
        })
    }

    /// Create a `Filter` that requires the `LazyReqStore` is already filled.
    pub fn filled() -> impl Filter<Extract = (), Error = Rejection> {
        store().and_then(|store| {
            if store.filled() {
                future::ok(())
            } else {
                future::err(reject::reject())
            }
        })
    }

    /// Create a `Filter` that extracts stored `Request`.
    pub fn stored_req() -> impl Filter<Extract = (Request,), Error = Rejection> {
        get_store().and_then(|store| future::ready(store.borrow().cloned().ok_or(reject::reject())))
    }
}
