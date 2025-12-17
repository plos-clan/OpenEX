use crate::compiler::ast::vm_ir::{ByteCode, ConstantTable, IrFunction, Types};
use crate::compiler::file::SourceFile;
use crate::compiler::Compiler;
use crate::runtime::operation::{
    add_value, big_value, div_value, equ_value, less_value, mul_value, not_equ_value, not_value,
    self_add_value, self_sub_value, sub_value,
};
use crate::runtime::thread::{add_thread_join, OpenEXThread};
use crate::runtime::RuntimeError;
use smol_str::{SmolStr, ToSmolStr, format_smolstr};
use std::fmt::Display;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Float(f64),
    String(String),
    Ref(SmolStr),
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Float(x) => {
                // 避免科学计数法，保留合理精度
                if x.fract() == 0.0 {
                    write!(f, "{:.1}", x)
                } else {
                    write!(f, "{}", x)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Ref(r) => write!(f, "{}", r),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StackFrame {
    constant_table: ConstantTable,
    local_tables: Vec<Value>,
    op_stack: Vec<Value>,
    name: String,
    file_name: SmolStr,
    pc: usize, // 指令执行索引(仅当前栈帧)
    codes: Vec<ByteCode>,
    native: Option<SmolStr>, // 本地函数栈帧路径
    args: usize,             // 从父栈帧提取的参数个数
}

fn element_to_value((types, value): (Types, SmolStr)) -> Value {
    match types {
        Types::String => Value::String(value.to_string()),
        Types::Number => {
            Value::Int(value.parse::<i64>().unwrap())
        }
        Types::Float => {
            Value::Float(value.parse::<f64>().unwrap())
        }
        Types::Bool => Value::Bool(value == "true"),
        Types::Ref => Value::Ref(value.to_smolstr()),
        Types::Null => Value::Null,
    }
}

impl StackFrame {
    pub fn new(
        name: String,
        file_name: SmolStr,
        codes: Vec<ByteCode>,
        locals: usize,
        constant_table: ConstantTable,
        native: Option<SmolStr>,
        args: usize,
    ) -> StackFrame {
        Self {
            constant_table,
            name,
            file_name,
            op_stack: Vec::new(),
            local_tables: vec![Value::Null; locals],
            pc: 0,
            codes,
            native,
            args,
        }
    }

    pub fn set_var_table(&mut self, index: usize, value: Value) {
        self.local_tables[index] = value;
    }

    pub fn get_var_table(&self, index: usize) -> &Value {
        self.local_tables.get(index).unwrap()
    }

    pub fn is_native(&self) -> Option<SmolStr> {
        self.native.clone()
    }

    pub fn get_const(&self, index: usize) -> Option<Value> {
        if let Some(element) = self.constant_table.get_const(index) {
            return Some(element_to_value(element));
        }
        None
    }

    pub fn get_args(&self) -> usize {
        self.args
    }

    pub fn get_op_stack_top(&self) -> Option<Value> {
        self.op_stack.last().cloned()
    }

    pub fn pop_op_stack(&mut self) -> Option<Value> {
        self.op_stack.pop()
    }

    pub fn push_op_stack(&mut self, value: Value) {
        self.op_stack.push(value);
    }

    pub fn get_frame_name(&self) -> (&str, &str) {
        (&self.name, &self.file_name)
    }

    pub fn fetch_current(&self) -> Option<&ByteCode> {
        self.codes.get(self.pc)
    }

    pub fn next_pc(&mut self) {
        self.pc += 1;
    }

    #[allow(dead_code)] // TODO
    pub fn set_next_pc(&mut self, next_pc: usize) {
        self.pc = next_pc;
    }
}

#[derive(Clone)]
pub(crate) struct Executor {
    threads: Vec<OpenEXThread>,
    files: Vec<SourceFile>,
}

impl Executor {
    pub fn new() -> Executor {
        Self {
            threads: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn add_thread(&mut self, thread: OpenEXThread) -> &mut OpenEXThread {
        self.threads.push(thread);
        self.threads.last_mut().unwrap()
    }

    pub fn get_path_func(&mut self, path: &str) -> Option<(ConstantTable, IrFunction)> {
        let mut sp = path.split('/');
        let file = sp.next().unwrap();
        let func = sp.next().unwrap();

        for i in 0..self.files.len() {
            if self.files[i].name.split(".").next().unwrap() == file {
                let t = self.files.get_mut(i).unwrap();
                let r_table = t.get_vmir_table().unwrap();
                for ir_func in r_table.get_functions() {
                    if ir_func.name.as_str() == func {
                        return Some((r_table.get_constant_table(), ir_func));
                    }
                }
                break;
            }
        }
        None
    }

    pub fn get_first(&self) -> &SourceFile {
        for file in &self.files {
            if !file.is_library {
                return file;
            }
        }
        unreachable!()
    }

    pub fn run(mut self, mut compiler: Compiler) {
        self.files.append(compiler.get_files());
        let main_script = self.get_first();
        let vm_ir = main_script.ir_table.as_ref().unwrap().as_ref();
        let refs = Arc::new(RwLock::new(self.clone()));
        add_thread_join(
            refs,
            format!("{}:main", main_script.name).to_smolstr(),
            vm_ir,
            main_script.name.to_smolstr(),
        );
    }
}

pub(crate) fn run_executor(
    frame_index: usize,
    executor: &mut Executor,
    thread: &mut OpenEXThread,
) -> Result<(Option<StackFrame>, bool), RuntimeError>
// Option<StackFrame> 不为 None 代表是函数调用请求
// bool 为 true 代表操作栈栈顶作为返回值压入父栈帧
{
    let mut root_frame;
    let stack_frame;
    if frame_index == 0 {
        root_frame = None;
        stack_frame = thread.get_mut_frame(frame_index);
    } else {
        let call_stack = &mut thread.call_stack;
        let (left, right) = call_stack.split_at_mut(frame_index);
        root_frame = Some(&mut left[0]);
        stack_frame = &mut right[0];
    }

    while let Some(instr) = stack_frame.fetch_current() {
        match instr {
            ByteCode::Push(const_index0) => {
                let const_index = *const_index0;

                if let Some(value) = stack_frame.get_const(const_index) {
                    if let Value::Ref(path) = &value
                        && path.as_str() == "this"
                    {
                        let path = stack_frame.file_name.split(".").next().unwrap();
                        stack_frame.push_op_stack(Value::Ref(path.to_smolstr()));
                    } else {
                        stack_frame.push_op_stack(value);
                    }
                }
                stack_frame.next_pc();
            }
            ByteCode::LoadGlobal(var_index) => {
                let index = *var_index;
                let result = stack_frame.pop_op_stack().unwrap();
                if let Some(ref mut root) = root_frame {
                    root.set_var_table(index, result);
                } else {
                    stack_frame.set_var_table(index, result);
                }
                stack_frame.next_pc();
            }
            ByteCode::StoreGlobal(var_index) => {
                let index = *var_index;

                let result = if let Some(ref root) = root_frame {
                    root.get_var_table(index)
                } else {
                    stack_frame.get_var_table(index)
                };
                stack_frame.push_op_stack(result.clone());
                stack_frame.next_pc();
            }
            ByteCode::Load(var_index) => {
                let index = *var_index;
                let result = stack_frame.pop_op_stack().unwrap();
                stack_frame.set_var_table(index, result);
                stack_frame.next_pc();
            }
            ByteCode::Store(var_index) => {
                let value = stack_frame.get_var_table(*var_index);
                stack_frame.push_op_stack(value.clone());
                stack_frame.next_pc();
            }
            ByteCode::Add => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(add_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Sub => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(sub_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Nol => stack_frame.next_pc(),
            ByteCode::GetRef => {
                let ref1 = stack_frame.pop_op_stack().unwrap();
                let ref2 = stack_frame.pop_op_stack().unwrap();
                if let Value::Ref(ref_top) = ref1
                    && let Value::Ref(ref_bak) = ref2
                {
                    let all_ref = format_smolstr!("{}/{}", ref_bak, ref_top);
                    stack_frame.push_op_stack(Value::Ref(all_ref));
                } else {
                    unreachable!()
                }
                stack_frame.next_pc();
            }
            ByteCode::Call => {
                let result = stack_frame.pop_op_stack().unwrap();
                return if let Value::Ref(path) = result {
                    let panic_path = path.clone();
                    if let Some(function) = executor.get_path_func(&path) {
                        let func = function.1;
                        let codes = func.clone_codes();
                        stack_frame.next_pc();
                        let native = match func.is_native {
                            true => Some(panic_path),
                            false => None,
                        };
                        Ok((
                            Some(StackFrame::new(
                                func.name.to_string(),
                                func.filename,
                                codes,
                                func.locals,
                                function.0,
                                native,
                                func.args,
                            )),
                            false,
                        ))
                    } else {
                        Err(RuntimeError::NoSuchFunctionException(panic_path))
                    }
                } else {
                    Err(RuntimeError::VMError)
                };
            }
            ByteCode::Mul => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(mul_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Div => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(div_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::SAdd => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(self_add_value(value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::SSub => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(self_sub_value(value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Jump(pc) => {
                let jpc = *pc;
                stack_frame.set_next_pc(jpc);
            }
            ByteCode::JumpTrue(pc) => {
                let jpc = *pc;
                let top = stack_frame.pop_op_stack().unwrap();
                if let Value::Bool(value) = top {
                    if value {
                        stack_frame.set_next_pc(jpc);
                    } else {
                        stack_frame.next_pc();
                    }
                } else {
                    unreachable!()
                }
            }
            ByteCode::JumpFalse(pc) => {
                let jpc = *pc;
                let top = stack_frame.pop_op_stack().unwrap();
                if let Value::Bool(value) = top {
                    if value {
                        stack_frame.next_pc();
                    } else {
                        stack_frame.set_next_pc(jpc);
                    }
                } else {
                    unreachable!()
                }
            }
            ByteCode::Big => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(big_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Less => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(less_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Equ => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(equ_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::NotEqu => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                let value_1 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(not_equ_value(value_1, value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Not => {
                let value_0 = stack_frame.pop_op_stack().unwrap();
                stack_frame.push_op_stack(not_value(value_0)?);
                stack_frame.next_pc();
            }
            ByteCode::Return => {
                return Ok((None, true));
            }
            _ => todo!(),
        }
    }
    Ok((None, false))
}
