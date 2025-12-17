use smol_str::SmolStr;

pub mod executor;
mod thread;
mod operation;
mod control_flow;
mod value_table;

#[allow(dead_code)] // TODO
#[derive(Debug)]
pub enum RuntimeError {
    NoSuchFunctionException(SmolStr),
    TypeException(SmolStr),
    PrecisionLoss(SmolStr),
    VMError,
}

