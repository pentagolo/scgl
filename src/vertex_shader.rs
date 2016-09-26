use std::borrow::Borrow;

use gl;
use WithMakeCurrent;

use ContextState;

use HandleBorrow;

use ShaderType;
use ShaderHandleSpecifier;
use ShaderHandle;
use ShaderFacade;
use Shader;

pub enum VertexShaderType {}
unsafe impl ShaderType for VertexShaderType {
    fn enum_val() -> gl::types::GLenum {
        gl::VERTEX_SHADER
    }
}
pub type VertexShaderHandleSpecifier = ShaderHandleSpecifier<VertexShaderType>;
pub type VertexShaderHandle = ShaderHandle<VertexShaderType>;
pub type VertexShaderFacade<HB: HandleBorrow<VertexShaderHandleSpecifier>, WMC: WithMakeCurrent> = ShaderFacade<VertexShaderType, HB, WMC>;
pub type VertexShader<WMC: WithMakeCurrent> = Shader<VertexShaderType, WMC>;
