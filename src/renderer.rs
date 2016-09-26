use gl;
use api;
use WithMakeCurrent;
use ArrayBufferRef;

struct Renderer<WMC: WithMakeCurrent> {
    with_make_current_: WMC,
};
impl<WMC: WithMakeCurrent> Renderer {
    pub fn draw<ABWMC: WithMakeCurrent>(&self, array_buffer_ref: ArrayBufferRef<ABWMC>, offset: usize, stride: usize, len: usize, program_borrow: &ProgramBorrow) -> Result<(), Error> {
        with_make_current.with_make_current(|current_context|{
            if !array_buffer_borrow.is_same_sharing_group_as_current() {
                panic("ArrayBuffer is not shared")
            }
            if !program_borrow.is_same_sharing_group_as_current() {
                panic("Program is not shared")
            }
            unsafe {
                api::gl(*current_context.api_ptr()).DrawArrays(gl::TRIANGLES, )

            }
        })
    }
}
