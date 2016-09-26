use std::rc::Rc;
use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::borrow::{Borrow, BorrowMut};
use std::os::raw::c_void;
use std::mem::{forget, transmute};

use Error;
use gl;
use gl::types::GLuint;
use GlError;

use api;
use DynApi;

use CurrentContext;
use MakeCurrent;
use AlreadyCurrent;
use WithCurrentContext;
use WithMakeCurrent;

use ContextState;

use DynSharingGroup;

use VertexShader;
use FragmentShader;

use Handle;
use HandleSpecifier;
use HandleBorrow;
use HandleBorrowMut;

use std::cell::UnsafeCell;

pub struct ProgramHandleSpecifier;
impl HandleSpecifier for ProgramHandleSpecifier {
    type Raw = GLuint;
}

pub type ProgramHandle = Handle<ProgramHandleSpecifier>;

pub struct ProgramFacade<HB: HandleBorrow<ProgramHandleSpecifier>, WMC: WithMakeCurrent> {
    handle_borrow_: HB,
    with_make_current_: WMC,
}
impl<HB: HandleBorrow<ProgramHandleSpecifier>, WMC: WithMakeCurrent> ProgramFacade<HB, WMC> {
    pub unsafe fn already_current_ref<'s, 'cc>(&'s self, current_context: &'cc CurrentContext) -> ProgramFacade<&ProgramHandle, (&'cc CurrentContext, AlreadyCurrent)> {
        ProgramFacade {
            handle_borrow_: self.handle_borrow_.borrow(),
            with_make_current_: (current_context, AlreadyCurrent),
        }
    }
    pub fn gl_handle(&self) -> GLuint {
        self.handle_borrow_.borrow().raw()
    }
    /*
    pub fn attribute_location(&self, name: &str) -> Result<AttributeLocation, Error> {
        self.with_make_current_.with_make_current(|current_context| {
            unsafe {
                let name = CString::new(name).unwrap();
                let gl = api::gl(&*current_context.api_ptr());
                AttributeLocation { gl_handle_: gl.GetAttribLocation(self.gl_handle(), name.as_ptr()) }
            }
        }
    }
    */
}
impl<HB: HandleBorrow<ProgramHandleSpecifier>, WMC: WithMakeCurrent> ProgramFacade<HB, WMC> {
    pub unsafe fn already_current_mut<'s, 'cc>(&'s mut self, current_context: &'cc CurrentContext) -> ProgramFacade<&'s mut HB, (&'cc CurrentContext, AlreadyCurrent)> {
        ProgramFacade {
            handle_borrow_: self.handle_borrow_.borrow_mut(),
            with_make_current_: (current_context, AlreadyCurrent),
        }
    }
    pub fn setup_from_vertex_and_fragment_shader_src(&mut self, vertex_shader_src: &str, fragment_shader_src: &str) -> Result<(), Error> {
        self.handle_borrow_.borrow_mut();
        self.with_make_current_.with_make_current(|current_context| {
            unsafe {
                let gl = api::gl(&*current_context.api_ptr());
                let vertex_shader = try!(VertexShader::new_from_src((current_context, AlreadyCurrent), vertex_shader_src));
                let fragment_shader = try!(FragmentShader::new_from_src((current_context, AlreadyCurrent), fragment_shader_src));
                gl.AttachShader(self.gl_handle(), vertex_shader.gl_handle());
                gl.AttachShader(self.gl_handle(), fragment_shader.gl_handle());
                gl.LinkProgram(self.gl_handle());
                Ok(())
            }
        })
    }
}
pub struct Program<WMC: WithMakeCurrent> {
    program_facade_: ProgramFacade<ProgramHandle, WMC>,
}
impl<WMC: WithMakeCurrent> Deref for Program<WMC> {
    type Target = ProgramFacade<ProgramHandle, WMC>;
    fn deref(&self) -> &Self::Target {
        &self.program_facade_
    }
}
impl<WMC: WithMakeCurrent> DerefMut for Program<WMC> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.program_facade_
    }
}
impl<WMC: WithMakeCurrent> Program<WMC> {
    pub fn new_from_vertex_and_fragment_shader_src(with_make_current: WMC, vertex_shader_src: &str, fragment_shader_src: &str) -> Result<Self, Error> {
        with_make_current.with_make_current(|current_context| {
            unsafe {
                let mut program = Program {
                    program_facade_: ProgramFacade {
                        handle_borrow_: ProgramHandle::new(api::gl(&*current_context.api_ptr()).CreateProgram()),
                        with_make_current_: with_make_current.clone(),
                    },
                };
                try!(program.already_current_mut(current_context).setup_from_vertex_and_fragment_shader_src(vertex_shader_src, fragment_shader_src));
                Ok(program)
            }
        })
    }
}
impl<WMC: WithMakeCurrent> Drop for Program<WMC> {
    fn drop(&mut self) {
        self.program_facade_.with_make_current_.with_make_current(|current_context| {
            unsafe {
                // TODO: Check errors.
                api::gl(&*current_context.api_ptr()).DeleteProgram(self.gl_handle());
                Ok(())
            }
        });
    }
}

/*
#[derive(Copy, Clone)]
struct AttribLocation {
    gl_handle_: GlAttribLocationHandle,
}
impl AttribLocation {
    pub fn gl_handle() -> GlAttribLocationHandle {
        gl_handle_
    }
}

#[derive(Copy, Clone)]
pub struct VertexAttribType(gl::types::GLenum);
impl VertexAttribType {
    pub unsafe fn new(t: gl::types::GLenum) -> Self {
        VertexAttribType(t)
    }
    pub const BYTE: VertexAttribType = unsafe { VertexAttribType::new(gl::BYTE) };
    pub const UNSIGNED_BYTE: VertexAttribType = unsafe { VertexAttribType::new(gl::UNSIGNED_BYTE) };
    pub const SHORT: VertexAttribType = unsafe { VertexAttribType::new(gl::SHORT) };
    pub const UNSIGNED_SHORT: VertexAttribType = unsafe { VertexAttribType::new(gl::UNSIGNED_SHORT) };
    pub const INT: VertexAttribType = unsafe { VertexAttribType::new(gl::INT) };
    pub const UNSIGNED_INT: VertexAttribType = unsafe { VertexAttribType::new(gl::UNSIGNED_INT) };
    pub const HALF_FLOAT: VertexAttribType = unsafe { VertexAttribType::new(gl::HALF_FLOAT) };
    pub const FLOAT: VertexAttribType = unsafe { VertexAttribType::new(gl::FLOAT) };
    pub const DOUBLE: VertexAttribType = unsafe { VertexAttribType::new(gl::DOUBLE) };
    pub const FIXED: VertexAttribType = unsafe { VertexAttribType::new(gl::FIXED) };
    pub const INT_2_10_10_10_REV: VertexAttribType = unsafe { VertexAttribType::new(gl::INT_2_10_10_10_REV) };
    pub const UNSIGNED_INT_2_10_10_10_REV: VertexAttribType = unsafe { VertexAttribType::new(gl::UNSIGNED_INT_2_10_10_10_REV) };
    pub const UNSIGNED_INT_10F_11F_11F_REV: VertexAttribType = unsafe { VertexAttribType::new(gl::UNSIGNED_INT_10F_11F_11F_REV) };
}
impl Deref for VertexAttribType {
    type Target = gl::types::GLenum;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct VertexAttribSize(gl::types::GLint);
impl VertexAttribSize {
    pub unsafe fn new(t: gl::types::GLint) -> Self {
        VertexAttribType(t)
    }
    pub const _1: VertexAttribSize = unsafe { VertexAttribSize::new(1) };
    pub const _2: VertexAttribSize = unsafe { VertexAttribSize::new(2) };
    pub const _3: VertexAttribSize = unsafe { VertexAttribSize::new(3) };
    pub const _4: VertexAttribSize = unsafe { VertexAttribSize::new(4) };
}
impl Deref for VertexAttribSize {
    type Target = gl::types::GLenum;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct VertexAttrib {
    location_: AttribLocation,
    ArrayBuffer,
    type_: VertexAttribType,
    size_: VertexAttribSize,
    stride_: usize,
    offset_: usize,
}
impl VertexAttrib {
    pub fn new(location: AttribLocation, type: VertexAttribType, size: VertexAttribSize, stride: usize, offset: usize) -> Self {
        VertexAttribPtr {
            location_: location,
            type_: type,
            size_: size,
            stride_: stride,
            offset_: offset,
        }
    }
    pub fn location(&self) -> AttributeLocation { self.location_ }
    pub fn type(&self) -> VertexAttribType { self.type_ }
    pub fn size(&self) -> VertexAttribSize { self.size_ }
    pub fn stride(&self) -> usize { self.stride_ }
    pub fn offset(&self) -> usize { self.offset_ }
}
*/
