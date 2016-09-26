use std::rc::Rc;
use std::sync::Arc;
use std::os::raw::c_void;
use std::mem::{forget, size_of};
use std::marker::PhantomData;

use gl;
use gl::types::GLuint;

use Error;

use Api;
use CurrentContext;
use StateCache;

/// Target specific enumeration values and implementation state change optimization for buffers.
pub unsafe trait BufferTarget {
    /// Get the target enumeration value. This should not change.
    fn enum_val() -> gl::types::GLenum;
    /// Whether the buffer is marked as bound in the state cache.
    fn marked_bound(&StateCache) -> GLuint;
    /// Mark the buffer as bound in the state cache.
    unsafe fn mark_bound(&StateCache, GLuint);
}

/// A buffer which implements Sync and Send.
pub struct AsyncBuffer<Target: BufferTarget> {
    gl_handle_: GLuint,
    api_: Arc<Api>,
    phantom_target_: PhantomData<Target>,
}
impl<Target: BufferTarget> AsyncBuffer<Target> {
    /// Get the api
    pub fn api(&self) -> &Arc<Api> {
        &self.api_
    }
    /// Get the opengl handle.
    pub fn gl_handle(&self) -> GLuint {
        self.gl_handle_
    }
    /// Unsafe split the async buffer into the handle and the api.
    pub unsafe fn split(mut self) -> (GLuint, Arc<Api>) {
        let res = (self.gl_handle_, self.api().clone());
        self.gl_handle_ = 0;
        res
    }
    /// Convert to the async buffer to a current buffer.
    pub fn to_current(self, current_context: Rc<CurrentContext>) -> CurrentBuffer<Target> {
        unsafe {
            let (gl_handle, api) = self.split();
            Target::mark_bound(current_context.state_cache(), 0);
            CurrentBuffer::from_gl_handle(gl_handle, current_context)
        }
    }
    /// Unsafe create async buffer from gl handle.
    pub unsafe fn from_gl_handle(gl_handle: GLuint, api: Arc<Api>) -> Self {
        AsyncBuffer {
            gl_handle_: gl_handle,
            api_: api,
            phantom_target_: PhantomData,
        }
    }
}
impl<Target: BufferTarget> Drop for AsyncBuffer<Target> {
    fn drop(&mut self) {
        if self.gl_handle() == 0 {
            panic!("Drop of AsyncBuffer")
        }
    }
}

/// A buffer which does not implement Sync or Send, but may be actually used.
pub struct CurrentBuffer<Target: BufferTarget> {
    gl_handle_: GLuint,
    current_context_: Rc<CurrentContext>,
    phantom_target_: PhantomData<Target>,
}
impl<Target: BufferTarget> CurrentBuffer<Target> {
    /// Get the gl handle.
    pub fn gl_handle(&self) -> GLuint {
        self.gl_handle_
    }
    /// Get the current context.
    pub fn current_context(&self) -> &Rc<CurrentContext> {
        &self.current_context_
    }
    /// Bind the buffer.
    fn bind(&self, current_context: &CurrentContext) -> Result<(), Error> {
        unsafe {
            if Target::marked_bound(current_context.state_cache()) != self.gl_handle() {
                // TODO: Error checking.
                current_context.gl().BindBuffer(Target::enum_val(), self.gl_handle());
                Target::mark_bound(current_context.state_cache(), self.gl_handle());
            }
            Ok(())
        }
    }
    /// Set the data of the buffer.
    fn set_data<DataElem: Copy>(&self, current_context: &CurrentContext, data: &[DataElem]) -> Result<(), Error> {
        unsafe {
            try!(self.bind(current_context));
            let data_len = (size_of::<DataElem>() * data.len()) as isize;
            let data_ptr = data as *const [DataElem] as *const c_void;
            // TODO: Error checking.
            current_context.gl().BufferData(Target::enum_val(), data_len, data_ptr, gl::STATIC_DRAW);
            Ok(())
        }
    }
    /// Unsafe split the current buffer into the handle and the current context.
    pub unsafe fn split(mut self) -> (GLuint, Rc<CurrentContext>) {
        let res = (self.gl_handle_, self.current_context_.clone());
        if Target::marked_bound(self.current_context().state_cache()) == self.gl_handle() {
            Target::mark_bound(self.current_context().state_cache(), 0);
        }
        self.gl_handle_ = 0;
        res
    }
    /// Unsafe convert to the current buffer to an async buffer. It is unsafe because glFinish has
    /// to be called before.
    pub unsafe fn to_async(self) -> AsyncBuffer<Target> {
        let (gl_handle, current_context) = self.split();
        AsyncBuffer::from_gl_handle(gl_handle, current_context.api().clone())
    }
    /// Unsafe create current buffer from gl handle.
    pub unsafe fn from_gl_handle(gl_handle: GLuint, current_context: Rc<CurrentContext>) -> Self {
        CurrentBuffer {
            gl_handle_: gl_handle,
            current_context_: current_context,
            phantom_target_: PhantomData,
        }
    }
    /// Create a new current buffer.
    pub fn create(current_context: Rc<CurrentContext>, state_cache: &mut StateCache) -> Result<Self, Error> {
        unsafe {
            let mut gl_handle: GLuint = 0;
            // TODO: Error checking.
            current_context.gl().GenBuffers(1, &mut gl_handle as *mut GLuint);
            Ok(Self::from_gl_handle(gl_handle, current_context))
        }
    }
}
impl<Target: BufferTarget> Drop for CurrentBuffer<Target> {
    fn drop(&mut self) {
        unsafe {
            if self.gl_handle() != 0 {
                if Target::marked_bound(self.current_context().state_cache()) == self.gl_handle() {
                    Target::mark_bound(self.current_context().state_cache(), 0);
                }
                // TODO: Error checking.
                self.current_context().gl().DeleteBuffers(1, &self.gl_handle() as *const GLuint);
            }
        }
    }
}




/*
pub struct AsyncBuffer {}

pub struct BufferHandleSpecifier<T: BufferTarget>(PhantomData<T>);
impl<T: BufferTarget> HandleSpecifier for BufferHandleSpecifier<T> {
    type Raw = GLuint;
}

pub type BufferHandle<T: BufferTarget> = Handle<BufferHandleSpecifier<T>>;

pub struct BufferFacade<T: BufferTarget, HB: HandleBorrow<BufferHandleSpecifier<T>>, WMC: WithMakeCurrent> {
    handle_borrow_: HB,
    with_make_current_: WMC,
    phantom_target_: PhantomData<T>,
}
impl<T: BufferTarget, HB: HandleBorrow<BufferHandleSpecifier<T>>, WMC: WithMakeCurrent> BufferFacade<T, HB, WMC> {
    pub unsafe fn already_current_ref<'s, 'cc>(&'s self, current_context: &'cc CurrentContext) -> BufferFacade<T, &BufferHandle<T>, (&'cc CurrentContext, AlreadyCurrent)> {
        BufferFacade {
            handle_borrow_: self.handle_borrow_.borrow(),
            with_make_current_: (current_context, AlreadyCurrent),
            phantom_target_: PhantomData,
        }
    }
    pub unsafe fn with_make_already_current<R, F: FnOnce(&CurrentContext, &BufferFacade<T, &BufferHandle<T>, (&CurrentContext, AlreadyCurrent)>) -> Result<R, Error>>(&self, f: F) -> Result<R, Error> {
        let handle: &BufferHandle<T> = self.handle_borrow_.borrow();
        self.with_make_current_.with_make_current(|current_context|{
            let already_current: BufferFacade<T, _, _> = BufferFacade {
                handle_borrow_: handle,
                with_make_current_: (current_context, AlreadyCurrent),
                phantom_target_: PhantomData,
            };
            f(current_context, &already_current)
        })
    }
    pub fn gl_handle(&self) -> GLuint {
        self.handle_borrow_.borrow().raw()
    }
    pub fn is_marked_bound(&self) -> Result<bool, Error> {
        unsafe {
            self.with_make_already_current(|current_context, already_current| {
                Ok(T::marked_bound(&*current_context.state().get()) == already_current.gl_handle())
            })
        }
    }
    pub fn bind(&self) -> Result<(), Error> {
        unsafe {
            self.with_make_already_current(|current_context, already_current| {
                if !try!(already_current.is_marked_bound()) {
                    // TODO: Error checking.
                    api::gl(&*current_context.api_ptr()).BindBuffer(T::enum_val(), self.gl_handle());
                    T::mark_bound(self.gl_handle(), &mut *current_context.state().get());
                    Ok(())
                } else {
                    Ok(())
                }
            })
        }
    }
    pub fn unbind(&self) -> Result<(), Error> {
        unsafe {
            self.with_make_already_current(|current_context, already_current| {
                if try!(already_current.is_marked_bound()) {
                    // TODO: Error checking.
                    T::mark_bound(0 as gl::types::GLenum, &mut *current_context.state().get());
                }
                api::gl(&*current_context.api_ptr()).BindBuffer(T::enum_val(), 0);
                Ok(())
            })
        }
    }
}
impl<T: BufferTarget, HB: HandleBorrowMut<BufferHandleSpecifier<T>>, WMC: WithMakeCurrent> BufferFacade<T, HB, WMC> {
    pub unsafe fn already_current_mut<'s, 'cc>(&'s mut self, current_context: &'cc CurrentContext) -> BufferFacade<T, &'s mut BufferHandle<T>, (&'cc CurrentContext, AlreadyCurrent)> {
        BufferFacade {
            handle_borrow_: self.handle_borrow_.borrow_mut(),
            with_make_current_: (current_context, AlreadyCurrent),
            phantom_target_: PhantomData,
        }
    }
    pub unsafe fn with_make_already_current_mut<R, F: FnOnce(&CurrentContext, &mut BufferFacade<T, &mut BufferHandle<T>, (&CurrentContext, AlreadyCurrent)>) -> Result<R, Error>>(&mut self, f: F) -> Result<R, Error> {
        let handle_mut: &mut BufferHandle<T> = self.handle_borrow_.borrow_mut();
        self.with_make_current_.with_make_current(|current_context|{
            let mut already_current: BufferFacade<T, _, _> = BufferFacade {
                handle_borrow_: handle_mut,
                with_make_current_: (current_context, AlreadyCurrent),
                phantom_target_: PhantomData,
            };
            f(current_context, &mut already_current)
        })
    }
    pub fn set_data<D: Copy>(&mut self, data: &[D]) -> Result<(), Error> {
        unsafe {
            self.with_make_already_current_mut(|current_context, already_current| {
                try!(already_current.bind());
                // TODO: Let the client specify the usage.
                // TODO: Error checking
                let data_len = (size_of::<D>() * data.len()) as isize;
                let data_ptr = data as *const [D] as *const c_void;
                api::gl(&*current_context.api_ptr()).BufferData(T::enum_val(), data_len, data_ptr, gl::STATIC_DRAW);
                Ok(())
            })
        }

    }
}

pub struct Buffer<T: BufferTarget, WMC: WithMakeCurrent> {
    buffer_facade_: BufferFacade<T, BufferHandle<T>, WMC>,
}
impl<T: BufferTarget, WMC: WithMakeCurrent> Deref for Buffer<T, WMC> {
    type Target = BufferFacade<T, BufferHandle<T>, WMC>;
    fn deref(&self) -> &Self::Target {
        &self.buffer_facade_
    }
}
impl<T: BufferTarget, WMC: WithMakeCurrent> DerefMut for Buffer<T, WMC> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer_facade_
    }
}
impl<T: BufferTarget, WMC: WithMakeCurrent> Buffer<T, WMC> {
    pub fn new_with_data<D: Copy>(with_make_current: WMC, data: &[D]) -> Result<Self, Error> {
        with_make_current.with_make_current(|current_context| {
            unsafe {
                let mut gl_handle: GLuint = 0;
                api::gl(&*current_context.api_ptr()).GenBuffers(1, &mut gl_handle as *mut GLuint);
                let mut buf: Buffer<T, _> = Buffer {
                    buffer_facade_: BufferFacade {
                        handle_borrow_: BufferHandle::new(gl_handle),
                        with_make_current_: with_make_current.clone(),
                        phantom_target_: PhantomData,
                    },
                };
                try!(buf.already_current_mut(current_context).set_data(data));
                Ok(buf)
            }
        })
    }
}
impl<T: BufferTarget, WMC: WithMakeCurrent> Drop for Buffer<T, WMC> {
    fn drop(&mut self) {
        self.buffer_facade_.with_make_current_.with_make_current(|current_context| {
            unsafe {
                // TODO: Check errors.
                api::gl(&*current_context.api_ptr()).DeleteBuffers(1, &self.gl_handle() as *const GLuint);
                Ok(())
            }
        });
    }
}
*/

/*
pub struct RawBuffer<T: BufferTarget> {
    gl_handle_: GLuint,
    phantom_target_: PhantomData,
}
impl RawBuffer {
    pub unsafe fn new_from_gl_handle(gl_handle: GLuint) -> Self {
        debug_assert!(gl_handle != 0);
        RawBuffer {
            gl_handle_: gl_handle,
            phantom_target_: PhantomData,
        }
    }
    pub unsafe fn already_current_new(gl: &Gl) -> Result<Self, Error> {
        // TODO: Error handling.
        let mut gl_handle: GlBufferHandle = 0;
        gl.GenBuffers(1, &mut gl_handle as *mut GlBufferHandle);
        let rb = Self::new_from_gl_handle(gl_handle)
        Ok(rb)
    }
    pub unsafe fn already_current_new_with_data<E: Copy>(gl: &Gl, context_state: &ContextState, data: &[E]) -> Result<Self, Error> {
        let mut rb = try!(already_current_new(gl));
        try!(rb.already_current_set_data(gl, context_state, data));
        Ok(rb)
    }
    pub fn gl_handle(&self) -> GLuint {
        self.gl_handle_
    }
    pub fn is_marked_bound(&self, context_state: &ContextState) -> bool {
        T::marked_bound(context_state) == self.gl_handle()
    }
    pub unsafe fn already_current_bind(&self, gl: &Gl, context_state: &mut ContextState) -> Result<(), Error> {
        if !self.is_marked_bound(context_state) {
            // TODO: Error checking.
            gl.BindBuffer(T::enum_val(), self.gl_handle());
            T::mark_bound(self.gl_handle(), context_state);
            Ok(())
        } else {
            Ok(())
        }
    }
    pub unsafe fn already_current_unbind(&self, gl: &Gl, context_state: &mut ContextState) -> Result<(), Error> {
        if self.is_marked_bound(context_state) {
            T::mark_bound(0 as gl::types::GLenum, context_state);
        }
        // TODO: Error checking.
        api::gl(&*current_context.api_ptr()).BindBuffer(T::enum_val(), 0);
        Ok(())
    }
    pub unsafe fn already_current_set_data<E: Copy>(&mut self, gl: &Gl, context_state: &mut ContextState, data: &[E]) -> Result<(), Error> {
        try!(self.bind(gl, context_state));
        let data_len = (size_of::<D>() * data.len()) as isize;
        let data_ptr = data as *const [D] as *const c_void;
        // TODO: Let the client specify the usage.
        // TODO: Error checking
        gl.BufferData(T::enum_val(), data_len, data_ptr, gl::STATIC_DRAW);
        Ok(())
    }
    pub unsafe fn already_current_delete(self, gl: &Gl, context_state: &mut ContextState) -> Result<(), Error> {
        if self.is_marked_bound(context_state) {
            T::mark_bound(0 as gl::types::GLenum, context_state);
        }
        // TODO: Check errors.
        gl.DeleteBuffers(1, &self.gl_handle_ as *const GlBufferHandle);
        forget(self);
        Ok(())
    }
}
impl Drop for RawBuffer {
    panic!("Drop called for raw buffer")
}
*/
