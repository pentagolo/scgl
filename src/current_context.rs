use std::os::raw::c_void;
use std::sync::Arc;
use std::rc::{Rc, Weak};
use std::cell::UnsafeCell;

use Error;
use gl;
use gl::Gl;
use Api;
use Context;
use StateCache;


/// Whether an api exists.
/// This is used to prevent two or more apis to exist at the same time.
thread_local! {
    static CURRENT_CONTEXT_: UnsafeCell<Weak<CurrentContext>> = UnsafeCell::new(Weak::new());
}

pub struct CurrentContext {
    context_: UnsafeCell<Rc<Context>>,
    api_: Arc<Api>,
    state_cache_: StateCache,
}
impl CurrentContext {
    pub fn state_cache(&self) -> &StateCache {
        &self.state_cache_
    }
    pub fn context(&self) -> &Rc<Context> {
        unsafe { &*self.context_.get() }
    }
    pub fn api(&self) -> &Arc<Api> {
        &self.api_
    }
    pub fn gl(&self) -> &Gl {
        self.api().gl()
    }
}

pub fn make_current(context: Rc<Context>) -> Result<Rc<CurrentContext>, Error> {
    unsafe {
        CURRENT_CONTEXT_.with(|thread_local_current_context| {
            let current_context = match (*thread_local_current_context.get()).upgrade() {
                None => {
                    let api = context.api().clone();
                    let current_context = Rc::new(CurrentContext {
                        context_: UnsafeCell::new(context),
                        api_: api,
                        state_cache_: StateCache::new(),
                    });
                    *thread_local_current_context.get() = Rc::downgrade(&current_context);
                    current_context
                },
                Some(current_context) => {
                    *current_context.context_.get() = context;
                    current_context.state_cache_.clear();
                    current_context
                },
            };
            if !current_context.context().backend().is_current() {
                try!(current_context.context().backend().make_current());
            }
            Ok(current_context)
        })
    }
}
impl Drop for CurrentContext {
    fn drop(&mut self) {
        unsafe {
            self.api().backend().clear_current_context().unwrap();
        }
    }
}

pub struct CurrentRenderContext(pub CurrentContext);
impl CurrentRenderContext {
}


/*
use Error;
use Api;
use DynSharingGroup;
use DynContext;
use ContextState;
use create_context;

/// Thread local singleton which handles context switching.
/// Additionally it caches the state of the context, to enable opengl state change optimizations.
/// Multiple contexts only know the same opengl-object, if they are shared contexts.
/// Most objects (indicated by the name 'Active') of this library require that any context, which
/// knows about them, can be made current any time in their lifetime, so opengl-operations using
/// their associated opengl-object with a context may be performed.
/// A strong reference (Rc) to this thread local singleton indicates this dependency.
/// A thread local weak pointer provides the singleton functionality.
/// The singleton only exists if any strong references exist.
/// If the singleton exists a shared context can be made current.
/// If the singleton does not exist, no context can be made current.
/// Switching the current shared context to another shared one will not change the singleton
/// instance. It will only change the internal refernce to the current context and make the
/// opengl-context current.
/// Switching the current context (not null) to another which do not share is not allowed.
/// If no context is current, any context can be made current.
/// The current opengl-context will be set to null when the last strong reference will be
/// destroyed.
/// So changing the current context to another which do not share requires the destrcution of all
/// strong references to the thread local singleton.
/// There is the possibility that another library exposes safe functionality to change the current
/// opengl-context not using the mechanism provided here. So one has to call
/// ActiveContext::make_current(&self) before any opengl-operations depending on the context may
/// be done. This method will check whether the required context is the current context. If not,
/// it will discard any cached state change information and make it the current opengl-context.
/// Checking whether the required opengl-context is current may be tweeked.
/// If you know that your application does not use any other libraries which change the current
/// opengl-context without using this mechanism, you can provide the

/// Parts of the context which are cached.
struct ContextCache_ {
    /// The context itself.
    context: Rc<Context>,
    /// Cache of the api of the context.
    api: Rc<Api>,
}
impl ContextCache_ {
    /// Create a new context cache from a context.
    pub fn new(context: Rc<Context>) -> Self {
        let api = context.api().clone();
        ContextCache {
            context: context,
            api: api,
        }
    }
}


/// An active context represents the last context made current using this library.
/// Its main purpose is to cache context related state in order to speedup the safe usage of
/// OpenGL.
/// There may exist at most one instance of this struct for each thread. It is not Sync or Send.
/// Making a context active means
/// Its existance indicates that there is a context
/// It stores a strong reference (Rc) to the context made current using this library. So functions
/// of this library could make it current again before executing native opengl calls, in case the
/// current context was potentially changed without of using this api.
/// If there is no context
///
///
/// OpenGL internally uses a thread-local variable pointing to the current context (see
/// https://www.opengl.org/wiki/OpenGL_Context) which may be changed and is used implicitely by
/// most native opengl functions. An OpenGL context is an implementation of a complex state
/// machine which drives the rendering on the video hardware.
/// The existance of an instance of this struct in a thread means that an opengl context could be
/// made current, so functions of other parts of this library could safely call native opengl
/// functions which depend on a context being current.
/// It contains a strong reference to the OpenGL context wrapper struct provided by this library
/// which will be made current if needed. opengl context,
/// before other functions of this library may call native opengl functions.
/// Other libraries may expose functionality to change the current opengl conetxt without of using
/// the api provided here. So in general, when unknown code is executed, functions of this library
/// have to ensure
pub struct CurrentContext {
    /// Whether
    expect_context_not_changed_externally_: Cell<bool>,
    context_cache_: UnsafeCell<ContextCache_>,
    state_: UnsafeCell<ContextState>,
    phantom_context_: PhantomData<Rc<Context>>,
    reference_count_: Cell<usize>,
}
impl CurrentContext {
    fn new() -> Self {
        CurrentContext {
            expect_context_not_changed_externally_: Cell::new(false),
            context_cache_: UnsafeCell::new(ContextCache::new(None)),
            state_: UnsafeCell::new(ContextState::new(false)),
            phantom_context_: PhantomData,
        }
    }

    /// Get whether it can be expected that the context may not be changed externally.
    pub fn expect_context_not_changed_externally(&self) -> bool {
        self.expect_context_not_changed_externally_.get()
    }
    /// Clear that it can be expected that the context may not be changed externally.
    pub fn clear_expect_context_not_changed_externally(&self) {
        self.expect_context_not_changed_externally_.set(false);
    }
    /// Set that it can be expected that the context may not be changed externally.
    pub unsafe fn set_expect_context_not_changed_externally(&self) {
        self.expect_context_not_changed_externally_.set(true);
    }

    /// Get the sharing group of the least context made current.
    pub fn sharing_group_ptr(&self) -> *const DynSharingGroup {
        unsafe {
            (*self.context_cache_.get()).sharing_group
        }
    }
    /// Get the api of the least context made current.
    pub fn api_ptr(&self) -> *const DynApi {
        unsafe {
            (*self.context_cache_.get()).api
        }
    }

    /// Get the state.
    pub fn state(&self) -> &UnsafeCell<ContextState> { &self.state_ }
    /// Make the context current. Has to be called before any opengl calls are made.
    pub fn make_context_current(&self, context: &Rc<DynContext>) -> Result<(), Error> {
        unsafe {
            if
                &**context as *const DynContext == self.context_ptr() &&
                self.expect_context_not_changed_externally()
            {
                debug_assert!((*self.context_ptr()).backend().is_current());
                Ok(())
            } else {
                *self.context_cache_.get() = ContextCache::new(Some(context.clone()));
                // TODO: Update state
                if (*self.context_ptr()).backend().is_current() {
                    Ok(())
                } else {
                    (*self.context_ptr()).backend().make_current()
                }
            }
        }
    }
    /// Make a context with same sharing group current. Has to be called before any opengl calls
    /// are made.
    pub fn make_sharing_group_current(&self, sharing_group: &Arc<DynSharingGroup>) -> Result<(), Error> {
        unsafe {
            if
                &**sharing_group as *const DynSharingGroup == self.sharing_group_ptr() &&
                self.expect_context_not_changed_externally()
            {
                debug_assert!((*self.context_ptr()).backend().is_current());
                Ok(())
            } else {
                let context = try!(create_context(sharing_group));
                // TODO: Update state
                *self.context_cache_.get() = ContextCache::new(Some(context));
                (*self.context_ptr()).backend().make_current()
            }
        }
    }
    pub fn clear_current(&self) {
        unsafe {
            *self.context_cache_.get() = ContextCache::new(None);
            *self.state_.get() = ContextState::new(false);
        }
    }
}

/// The thread local weak reference which provides the singleton functionality.
thread_local!(pub static CURRENT_CONTEXT: Rc<CurrentContext> = Rc::new(CurrentContext::new()));
/// Get an rc to the current context.
pub fn current_context() -> Rc<CurrentContext> {
    CURRENT_CONTEXT.with(|cc| { cc.clone() })
}

/// Call a function with the current context as parameter.
pub trait WithCurrentContext: Clone {
    fn with_current_context<R, F: FnOnce(&CurrentContext) -> R>(&self, F) -> R;
}
/// Use the thread local current context as parameter to with_current_context.
#[derive(Clone)]
pub struct ThreadLocalCurrentContext {}
impl WithCurrentContext for ThreadLocalCurrentContext {
    fn with_current_context<R, F: FnOnce(&CurrentContext) -> R>(&self, f: F) -> R {
        CURRENT_CONTEXT.with(|current_context|{f(&**current_context)})
    }
}
impl WithCurrentContext for Rc<CurrentContext> {
    fn with_current_context<R, F: FnOnce(&CurrentContext) -> R>(&self, f: F) -> R {
        f(&*self)
    }
}
impl<'cc> WithCurrentContext for &'cc CurrentContext {
    fn with_current_context<R, F: FnOnce(&CurrentContext) -> R>(&self, f: F) -> R {
        f(*self)
    }
}

/// A trait to make the context or sharing-group current.
pub unsafe trait MakeCurrent: Clone {
    fn make_current(&self, &CurrentContext) -> Result<(), Error>;
    fn is_same_sharing_group_as_current(&self, &CurrentContext) -> bool;
}
/// Indicates that the context or sharing-group is already current.
/// Calling make_current results in a noop.
#[derive(Clone)]
pub struct AlreadyCurrent;
unsafe impl MakeCurrent for AlreadyCurrent {
    fn make_current(&self, _: &CurrentContext) -> Result<(), Error> {
        Ok(())
    }
    fn is_same_sharing_group_as_current(&self, _: &CurrentContext) -> bool {
        true
    }
}
/// Indicates that there is no context or sharing-group to make current.
/// Calling make_current will result in a panic.
#[derive(Clone)]
pub struct NothingCurrent;
unsafe impl MakeCurrent for NothingCurrent {
    fn make_current(&self, _: &CurrentContext) -> Result<(), Error> {
        panic!("Nothing to make current")
    }
    fn is_same_sharing_group_as_current(&self, _: &CurrentContext) -> bool {
        panic!("Nothing to compare sharing group with")
    }
}
unsafe impl MakeCurrent for Arc<DynSharingGroup> {
    fn make_current(&self, current_context: &CurrentContext) -> Result<(), Error> {
        current_context.make_sharing_group_current(self)
    }
    fn is_same_sharing_group_as_current(&self, current_context: &CurrentContext) -> bool {
        current_context.sharing_group_ptr() == &**self as *const DynSharingGroup
    }
}
unsafe impl<'sg> MakeCurrent for &'sg Arc<DynSharingGroup> {
    fn make_current(&self, current_context: &CurrentContext) -> Result<(), Error> {
        current_context.make_sharing_group_current(*self)
    }
    fn is_same_sharing_group_as_current(&self, current_context: &CurrentContext) -> bool {
        current_context.sharing_group_ptr() == &***self as *const DynSharingGroup
    }
}
unsafe impl MakeCurrent for Rc<DynContext> {
    fn make_current(&self, current_context: &CurrentContext) -> Result<(), Error> {
        current_context.make_context_current(self)
    }
    fn is_same_sharing_group_as_current(&self, current_context: &CurrentContext) -> bool {
        current_context.sharing_group_ptr() == &**self.sharing_group() as *const DynSharingGroup
    }
}
unsafe impl<'sg> MakeCurrent for &'sg Rc<DynContext> {
    fn make_current(&self, current_context: &CurrentContext) -> Result<(), Error> {
        current_context.make_context_current(*self)
    }
    fn is_same_sharing_group_as_current(&self, current_context: &CurrentContext) -> bool {
        current_context.sharing_group_ptr() == &**self.sharing_group() as *const DynSharingGroup
    }
}

pub unsafe trait WithMakeCurrent: Clone {
    fn with_make_current<R, F: FnOnce(&CurrentContext) -> Result<R, Error>>(&self, F) -> Result<R, Error>;
    fn with_make_current_mut<R, F: FnOnce(&CurrentContext) -> Result<R, Error>>(&mut self, f: F) -> Result<R, Error> {
        self.with_make_current(f)
    }
    fn is_same_sharing_group_as_current(&self, &CurrentContext) -> bool;
}
unsafe impl<WCC: WithCurrentContext, MC: MakeCurrent> WithMakeCurrent for (WCC, MC) {
    fn with_make_current<R, F: FnOnce(&CurrentContext) -> Result<R, Error>>(&self, f: F) -> Result<R, Error> {
        self.0.with_current_context(|current_context| {
            try!(self.1.make_current(current_context));
            f(current_context)
        })
    }
    fn is_same_sharing_group_as_current(&self, current_context: &CurrentContext) -> bool {
        self.1.is_same_sharing_group_as_current(current_context)
    }
}

/// Indicator whether the current context may be changed externally.
pub unsafe trait CurrentContextMayBeChangedExternally {
    /// Whether the current context may be changed externally.
    fn current_context_may_be_changed_externally(&self, current_context: &CurrentContext) -> bool;
}

/// Indicator that the current context was not changed externally.
pub struct UnsafeCurrentContextNotChangedExternally(());
impl UnsafeCurrentContextNotChangedExternally {
    /// Unsafely create indicator that the current context was not changed externally.
    pub unsafe fn new() -> Self {
        UnsafeCurrentContextWasNotChangedExternally(())
    }
}
impl CurrentContextMayBeChangedExternally for UnsafeCurrentContextNotChangedExternally {
    fn current_context_may_be_changed_externally(&self, current_context: &CurrentContext) {
        false
    }
}

/// Indicator that the current context was not changed externally.
pub struct DynamicCheckCurrentContextWasChangedExternally;
impl CurrentContextMayBeChangedExternally for DynamicCheckCurrentContextWasChangedExternally {
    fn current_context_may_be_changed_externally(&self, current_context: &CurrentContext) {
        current_context.expect_context_not_changed_externally_.get()
    }
}

*/
