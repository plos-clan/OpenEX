use smol_str::SmolStr;

pub mod executor;
mod thread;
mod operation;

#[allow(dead_code)] // TODO
#[derive(Debug)]
pub enum RuntimeError {
    NoSuchFunctionException(SmolStr),
    TypeException(SmolStr),
    VMError,
}

