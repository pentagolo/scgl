use gl;
use gl::types::GLuint;

use StateCache;

use BufferTarget;
use AsyncBuffer;
use CurrentBuffer;

pub enum ArrayBufferTarget {}
unsafe impl BufferTarget for ArrayBufferTarget {
    fn enum_val() -> gl::types::GLenum {
        gl::ARRAY_BUFFER
    }
    /// Whether the buffer is marked as bound in the state cache.
    fn marked_bound(state_cache: &StateCache) -> GLuint {
        state_cache.bound_array_buffer_gl_handle.get()
    }
    /// Mark the buffer as bound in the state cache.
    unsafe fn mark_bound(state_cache: &StateCache, gl_handle: GLuint) {
        state_cache.bound_array_buffer_gl_handle.set(gl_handle);
    }
}
pub type AsyncArrayBuffer = AsyncBuffer<ArrayBufferTarget>;
pub type CurrentArrayBuffer = CurrentBuffer<ArrayBufferTarget>;
