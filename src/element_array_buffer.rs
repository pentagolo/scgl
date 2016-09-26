use gl;
use gl::types::GLuint;

use StateCache;

use BufferTarget;
use AsyncBuffer;
use CurrentBuffer;

pub enum ElementArrayBufferTarget {}
unsafe impl BufferTarget for ElementArrayBufferTarget {
    fn enum_val() -> gl::types::GLenum {
        gl::ELEMENT_ARRAY_BUFFER
    }
    /// Whether the buffer is marked as bound in the state cache.
    fn marked_bound(state_cache: &StateCache) -> GLuint {
        state_cache.bound_element_array_buffer_gl_handle.get()
    }
    /// Mark the buffer as bound in the state cache.
    unsafe fn mark_bound(state_cache: &StateCache, gl_handle: GLuint) {
        state_cache.bound_element_array_buffer_gl_handle.set(gl_handle);
    }
}
pub type AsyncElementArrayBuffer = AsyncBuffer<ElementArrayBufferTarget>;
pub type CurrentElementArrayBuffer = CurrentBuffer<ElementArrayBufferTarget>;
