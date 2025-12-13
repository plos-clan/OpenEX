use smol_str::SmolStr;

pub mod executor;
mod thread;

#[allow(dead_code)] // TODO
#[derive(Debug)]
pub enum RuntimeError {
    NoSuchFunctionException(SmolStr),
    VMError,
}

