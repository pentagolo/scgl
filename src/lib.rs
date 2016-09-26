mod error;
pub use error::Error;

mod gl;
pub use gl::Gl;
pub use gl::Error as GlError;

mod api;
pub use api::ApiBackend;
pub use api::MakeApiBackend;
pub use api::Api;

mod context;
pub use context::ContextBackend;
pub use context::Context;

mod state_cache;
pub use state_cache::StateCache;

mod current_context;
pub use current_context::CurrentContext;
pub use current_context::make_current;

mod buffer;
pub use buffer::BufferTarget;
pub use buffer::AsyncBuffer;
pub use buffer::CurrentBuffer;

mod array_buffer;
pub use array_buffer::ArrayBufferTarget;
pub use array_buffer::AsyncArrayBuffer;
pub use array_buffer::CurrentArrayBuffer;

mod element_array_buffer;
pub use element_array_buffer::ElementArrayBufferTarget;
pub use element_array_buffer::AsyncElementArrayBuffer;
pub use element_array_buffer::CurrentElementArrayBuffer;

/*
mod shader;
pub use shader::ShaderType;
pub use shader::ShaderHandleSpecifier;
pub use shader::ShaderHandle;
pub use shader::ShaderFacade;
pub use shader::Shader;

mod vertex_shader;
pub use vertex_shader::VertexShaderType;
pub use vertex_shader::VertexShaderHandle;
pub use vertex_shader::VertexShaderFacade;
pub use vertex_shader::VertexShader;

mod fragment_shader;
pub use fragment_shader::FragmentShaderType;
pub use fragment_shader::FragmentShaderHandle;
pub use fragment_shader::FragmentShaderFacade;
pub use fragment_shader::FragmentShader;

mod program;
pub use program::ProgramHandleSpecifier;
pub use program::ProgramHandle;
pub use program::ProgramFacade;
pub use program::Program;
*/

//#[cfg(glfw)]
extern crate glfw;

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::raw::c_void;
    use std::mem::transmute;
    use std::sync::Arc;
    use std::rc::Rc;
    use std::io::{Write, stderr};
    use glfw;

    unsafe impl ApiBackend for glfw::Glfw {
        unsafe fn clear_current_context(&self) -> Result<(), Error> {
            self.make_context_current(None);
            Ok(())
        }
    }
    unsafe impl<'w> MakeApiBackend for &'w glfw::Window {
        unsafe fn get_proc_address(&mut self, name: &str) -> *const c_void {
            let res = unsafe { transmute(self.get_proc_address(name)) };
            let addr: usize = transmute(res);
            writeln!(&mut stderr(), "get_proc_address {}: {}", name, addr);
            res
        }
        unsafe fn into_backend(self) -> Box<ApiBackend> {
            Box::new(glfw::Glfw)
        }
    }
    struct GlfwContextBackend(pub Rc<glfw::Window>);
    unsafe impl ContextBackend for GlfwContextBackend {
        fn is_current(&self) -> bool {
            use glfw::Context;
            self.0.is_current();
        }
        unsafe fn make_current(&self) -> Result<(), Error> {
            use glfw::Context;
            self.0.make_current();
            Ok(())
        }
    }

    #[test]
    fn it_works() {
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        let (mut window, events) = glfw.create_window(300, 300, "Test", glfw::WindowMode::Windowed).unwrap();
        glfw::Context::make_current(&mut window);
        let api: Arc<Api> = { Arc::new(Api::new(&window)) };
        let window = Rc::new(window);
        let context: Rc<Context> = unsafe { Rc::new(Context::new(window, GlfwContextBackend(window))) };
        let current_context = make_current(context);
        /*
        let ab = ArrayBuffer::new_with_data((&*cc, c.clone()), &[
             1.0f32,  1.0f32,
            -1.0f32,  1.0f32,
            -1.0f32, -1.0f32,
             1.0f32, -1.0f32,
        ]).unwrap();
        let eab = ElementArrayBuffer::new_with_data((&*cc, c.clone()), &[
            0, 1, 2,
            2, 3, 0,
        ]).unwrap();
        let p = Program::new_from_vertex_and_fragment_shader_src(
            (&*cc, c.clone()),
            "",
            ""
        ).unwrap();*/
    }
}
