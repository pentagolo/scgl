use GlError;

use std::any::Any;
use std::fmt::Debug;

pub trait UnknownError: Any + Debug + 'static {}

#[derive(Debug)]
pub enum Error {
    ApiAlreadyExists,
    Gl(GlError),
    Unknown(Box<UnknownError>),
}
