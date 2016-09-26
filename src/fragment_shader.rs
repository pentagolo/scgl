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

pub enum FragmentShaderType {}
unsafe impl ShaderType for FragmentShaderType {
    fn enum_val() -> gl::types::GLenum {
        gl::FRAGMENT_SHADER
    }
}
pub type FragmentShaderHandleSpecifier = ShaderHandleSpecifier<FragmentShaderType>;
pub type FragmentShaderHandle = ShaderHandle<FragmentShaderType>;
pub type FragmentShaderFacade<HB: HandleBorrow<FragmentShaderHandleSpecifier>, WMC: WithMakeCurrent> = ShaderFacade<FragmentShaderType, HB, WMC>;
pub type FragmentShader<WMC: WithMakeCurrent> = Shader<FragmentShaderType, WMC>;
