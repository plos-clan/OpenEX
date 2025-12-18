use smol_str::SmolStr;

mod control_flow;
pub mod executor;
mod operation;
mod thread;
mod value_table;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum RuntimeError {
    NoSuchFunctionException(SmolStr), // 找不到函数
    TypeException(SmolStr),           // 类型检查错误
    PrecisionLoss(SmolStr),           // 精度转换损失
    VMError,                          // 解释器内部错误
}
