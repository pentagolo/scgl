use std::cell::Cell;
use gl::types::GLuint;

/// A state of a current context.
/// Records things like which buffers and textures are bound.
/// It is used to minimize the opengl state changes.
#[derive(Clone, Debug)]
pub struct StateCache {
    /// The handle of the bound element-buffer.
    pub bound_array_buffer_gl_handle: Cell<GLuint>,
    /// The handle of the bound index-buffer.
    pub bound_element_array_buffer_gl_handle: Cell<GLuint>,
    // ... TODO
}
impl StateCache {
    pub fn new() -> Self {
        StateCache {
            bound_array_buffer_gl_handle: Cell::new(0),
            bound_element_array_buffer_gl_handle: Cell::new(0),
        }
    }
    pub fn clear(&self) {
        self.bound_array_buffer_gl_handle.set(0);
        self.bound_element_array_buffer_gl_handle.set(0);
    }
}
