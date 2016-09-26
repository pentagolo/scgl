use Error;
use DynApi;
use DynContext;

use std::sync::Arc;
use std::rc::Rc;

/// Backend of a sharing group.
/// A sharing group is a group of contexts which share their objects.
/// It may be empty. In this case the implementations should internally own a context to guarantee
/// that the objects live on.
/// TODO: Make it sync.
pub unsafe trait SharingGroupBackend: 'static {
    /// Create a new context.
    unsafe fn create_context(&self) -> Result<Box<ContextBackend>, Error>;
}

/// Frontend of a sharing group.
/// A sharing group is a group of contexts which share their objects.
/// It may be empty. In this case the objects are guaranteed to live on,
pub struct SharingGroup {
    api_: Arc<Api>,
    backend_: Box<SharingGroupBackend>,
}
impl SharingGroup {
    pub unsafe fn new(api: Arc<Api>, backend: Box<SharingGroupBackend>) -> SharingGroup {
        SharingGroup {
            api_: api,
            backend_: backend,
        }
    }
    pub fn from_api(api: Arc<Api>) -> Self {
        let backend = api.backend().create_sharing_group();
        SharingGroup{
            api_: api,
            backend_: backend
        }
    }
    pub fn backend(&self) -> &Backend { &self.backend_ }
    pub fn api(&self) -> &Arc<DynApi> { &self.api_ }
}
