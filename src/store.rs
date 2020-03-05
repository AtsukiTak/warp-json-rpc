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
