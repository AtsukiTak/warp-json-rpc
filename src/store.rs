use crate::req::Request;
use futures::future;
use lazycell::AtomicLazyCell;
use std::sync::Arc;
use warp::{filters, reject, Filter, Rejection};

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

/// Create a `Filter` that extracts `LazyReqStore`.
pub(crate) fn store() -> impl Filter<Extract = (LazyReqStore,), Error = Rejection> + Copy {
    filters::ext::optional::<LazyReqStore>().and_then(|opt| {
        match opt {
            Some(store) => future::ok(store),
            None => {
                log::error!("Your Filter has to be wrapped by `JsonRpcService`");
                // TODO return custom error
                future::err(reject::reject())
            }
        }
    })
}

/// Create a `Filter` that requires the `LazyReqStore` is already filled.
pub(crate) fn filled() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    store()
        .and_then(|store: LazyReqStore| {
            if store.filled() {
                future::ok(())
            } else {
                future::err(reject::reject())
            }
        })
        .untuple_one()
}

/// Create a `Filter` that extracts stored `Request`.
pub(crate) fn stored_req() -> impl Filter<Extract = (Request,), Error = Rejection> + Copy {
    store().and_then(|store: LazyReqStore| {
        future::ready(store.borrow().cloned().ok_or_else(reject::reject))
    })
}
