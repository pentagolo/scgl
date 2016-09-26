use std::rc::Rc;
use std::sync::Arc;
use std::os::raw::c_void;
use std::mem::forget;

use gl;
use gl::types::GLuint;

use Error;

use Api;
use CurrentContext;
use StateCache;

pub unsafe trait ShaderType {
    /// Get the type enumeration value. This should not change.
    fn enum_val() -> gl::types::GLenum;
}

pub struct CurrentShader {
    gl_handle_: GLuint,
    current_context_: Rc<CurrentContext>,
}
impl CurrentShader {
    pub fn gl_handle(&self) -> GLuint
}

pub struct ShaderFacade<T: ShaderType, HB: HandleBorrow<ShaderHandleSpecifier<T>>, WMC: WithMakeCurrent> {
    handle_borrow_: HB,
    with_make_current_: WMC,
    phantom_target_: PhantomData<T>,
}
impl<T: ShaderType, HB: HandleBorrow<ShaderHandleSpecifier<T>>, WMC: WithMakeCurrent> ShaderFacade<T, HB, WMC> {
    pub unsafe fn already_current_ref<'s, 'cc>(&'s self, current_context: &'cc CurrentContext) -> ShaderFacade<T, &ShaderHandle<T>, (&'cc CurrentContext, AlreadyCurrent)> {
        ShaderFacade {
            handle_borrow_: self.handle_borrow_.borrow(),
            with_make_current_: (current_context, AlreadyCurrent),
            phantom_target_: PhantomData,
        }
    }
    pub fn gl_handle(&self) -> GLuint {
        self.handle_borrow_.borrow().raw()
    }
}
impl<T: ShaderType, HB: HandleBorrow<ShaderHandleSpecifier<T>>, WMC: WithMakeCurrent> ShaderFacade<T, HB, WMC> {
    pub unsafe fn already_current_mut<'s, 'cc>(&'s mut self, current_context: &'cc CurrentContext) -> ShaderFacade<T, &'s mut HB, (&'cc CurrentContext, AlreadyCurrent)> {
        ShaderFacade {
            handle_borrow_: self.handle_borrow_.borrow_mut(),
            with_make_current_: (current_context, AlreadyCurrent),
            phantom_target_: PhantomData,
        }
    }
    pub fn setup_from_src(&mut self, src: &str) -> Result<(), Error> {
        self.handle_borrow_.borrow_mut();
        self.with_make_current_.with_make_current(|current_context| {
            unsafe {
                let gl = api::gl(&*current_context.api_ptr());
                let ptr: *const *const gl::types::GLchar = transmute(&src.as_ptr() as *const *const u8);
                gl.ShaderSource(self.gl_handle(), 1, ptr, &(src.len() as gl::types::GLint) as *const gl::types::GLint);
                gl.CompileShader(self.gl_handle());
                Ok(())
            }
        })
    }
}
pub struct Shader<T: ShaderType, WMC: WithMakeCurrent> {
    shader_facade_: ShaderFacade<T, ShaderHandle<T>, WMC>,
}
impl<T: ShaderType, WMC: WithMakeCurrent> Deref for Shader<T, WMC> {
    type Target = ShaderFacade<T, ShaderHandle<T>, WMC>;
    fn deref(&self) -> &Self::Target {
        &self.shader_facade_
    }
}
impl<T: ShaderType, WMC: WithMakeCurrent> DerefMut for Shader<T, WMC> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shader_facade_
    }
}
impl<T: ShaderType, WMC: WithMakeCurrent> Shader<T, WMC> {
    pub fn new_from_src(with_make_current: WMC, src: &str) -> Result<Self, Error> {
        with_make_current.with_make_current(|current_context| {
            unsafe {
                let mut shader = Shader {
                    shader_facade_: ShaderFacade {
                        handle_borrow_: ShaderHandle::new(api::gl(&*current_context.api_ptr()).CreateShader(T::enum_val())),
                        with_make_current_: with_make_current.clone(),
                        phantom_target_: PhantomData,
                    },
                };
                try!(shader.already_current_mut(current_context).setup_from_src(src));
                Ok(shader)
            }
        })
    }
}
impl<T: ShaderType, WMC: WithMakeCurrent> Drop for Shader<T, WMC> {
    fn drop(&mut self) {
        self.shader_facade_.with_make_current_.with_make_current(|current_context| {
            unsafe {
                // TODO: Check errors.
                api::gl(&*current_context.api_ptr()).DeleteShader(self.gl_handle());
                Ok(())
            }
        });
    }
}





/*
pub struct ActiveShader<T: ShaderType> {
    gl_handle_borrow_: GlShaderHandle,
    actice_context_: Rc<ActiveContext>,
    phantom_type_: PhantomData<T>,
}
impl<T: ShaderType> Shader<T> {
    pub unsafe fn current_new(src: &str, ac: &Rc<ActiveContext>) -> Result<Self, Error> {
        let mut ac = ActiveShader{
            gl_handle_: T::create_shader(ac.api()),
            active_context_: ac.clone(),
            phantom_type_: PhantomData,
        }
        try!(ac.current_setup(src));
        Ok(ac)
    }
    pub fn new(src: &str, ac: &Rc<ActiveContext>) -> Result<Self, Error> {
        try!(ac.make_current());
        unsafe { Self::current_new(src, ac) }
    }
    pub unsafe fn current_setup(&mut self, src: &str) -> Result<(), Error> {
        // TODO: Error checking
        let gl = api::gl(active_context_.api().deref());
        gl.ShaderSource(self.gl_handle, 1, &src.as_ptr() as *const *const types::GLchar, &(src.len() as types::GLint) as *const types::GLint);
        gl.
        Ok(())
    }
    pub fn setup(&mut self, src: &str) -> Result<(), Error> {
        try!(self.active_context_.make_current());
        unsafe { self.current_setup(src) }
    }
}
*/
