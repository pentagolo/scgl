use Error;
use Api;

use std::sync::Arc;
use std::rc::Rc;

/// A context backend used by the api backend.
pub unsafe trait ContextBackend: 'static {
    /// Whether the context is the current context.
    fn is_current(&self) -> bool;
    /// Make the context current.
    /// A check whether the context is already the current context is not made before. So this
    /// optimization is expected to be done by the implementation.
    unsafe fn make_current(&self) -> Result<(), Error>;
}

/// Context used by the api.
pub struct Context {
    api_: Arc<Api>,
    backend_: Box<ContextBackend>,
}
impl Context {
    pub unsafe fn new(api: Arc<Api>, backend: Box<ContextBackend>) -> Self {
        Context {
            api_: api,
            backend_: backend,
        }
    }
    /// Get the api.
    pub fn api(&self) -> &Arc<Api> { &self.api_ }
    /// Get the backend.
    pub fn backend(&self) -> &ContextBackend { &*self.backend_ }
}
