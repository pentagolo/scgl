use std::os::raw::c_void;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, ATOMIC_BOOL_INIT, Ordering};

use gl;
use gl::Gl;
use Error;


/// Whether an api exists.
/// This is used to prevent two or more apis to exist at the same time.
static API_EXISTS_: AtomicBool = ATOMIC_BOOL_INIT;

/// Tries to set API_EXISTS_ on construction (fails if it is already set).
/// Clears API_EXISTS_ on destruction.
struct ScopedApiExistsGuard_(());
impl ScopedApiExistsGuard_ {
    fn new() -> Result<Self, Error> {
        if API_EXISTS_.swap(true, Ordering::AcqRel) {
            Err(Error::ApiAlreadyExists)
        } else {
            Ok(ScopedApiExistsGuard_(()))
        }
    }
}
impl Drop for ScopedApiExistsGuard_ {
    fn drop(&mut self) {
        API_EXISTS_.store(false, Ordering::AcqRel);
    }
}


/// Backend of an api.
/// An Api provides access to the procedure calls of an opengl api.
/// It may be shared between multiple contexts.
/// This trait will be used with dynamic dispatched only.
pub unsafe trait ApiBackend: Sync + 'static {
    /// Clear the current context.
    /// A check, whether the current context is cleared is not done before. So this optimization
    /// is expected to be done by the implementation.
    unsafe fn clear_current_context(&self) -> Result<(), Error>;
}

/// The creation of an api backend.
/// A context has to be current at the whole lifetime.
pub unsafe trait MakeApiBackend {
    /// Get the address of an opengl procedure.
    unsafe fn get_proc_address(&mut self, &str) -> *const c_void;
    /// Convert into the backend. No context need to be current afterwards.
    unsafe fn into_backend(self) -> Box<ApiBackend>;
}


/// Frontend for an api backend.
/// An Api provides access to the procedure calls of an opengl api.
/// There may only exists at most one api at time.
/// But note, there may exists more than one api backends at time.
/// Instances of this type will only be used by this library wrapped in an Arc<Api>.
/// Each context contains a strong reference to the api.
/// It is ensured that the Backend will be dropped when the api is dropped.
pub struct Api {
    gl_: gl::Gl,
    backend_: Box<ApiBackend>,
    scoped_api_exists_guard_: ScopedApiExistsGuard_,
}
impl Api {
    pub fn new<MAB: MakeApiBackend>(mut mab: MAB) -> Result<Self, Error> {
        let scoped_api_exists_guard = try!(ScopedApiExistsGuard_::new());
        unsafe {
            let gl = gl::Gl::load_with(|s| mab.get_proc_address(s));
            Ok(Api {
                gl_: gl,
                backend_: mab.into_backend(),
                scoped_api_exists_guard_: scoped_api_exists_guard,
            })
        }
    }
    // Get the backend.
    pub fn backend(&self) -> &ApiBackend { &*self.backend_ }
    // Get the opengl calls.
    pub fn gl(&self) -> &Gl { &self.gl_ }
}
