use crate::compiler::ast::vm_ir::{ByteCode, IrFunction};
use crate::compiler::file::SourceFile;
use crate::compiler::Compiler;
use crate::runtime::control_flow::{call_func, jump, jump_false, jump_true};
use crate::runtime::operation::{add_value, big_value, div_value, equ_value, get_ref, less_value, mul_value, not_equ_value, not_value, self_add_value, self_sub_value, sub_value};
use crate::runtime::thread::{add_thread_join, OpenEXThread};
use crate::runtime::value_table::{load_local, push_stack, store_local};
use crate::runtime::RuntimeError;
use smol_str::{SmolStr, ToSmolStr};
use std::fmt::Display;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone,PartialEq)]
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
            Self::Int(i) => write!(f, "{i}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Float(x) => {
                // 避免科学计数法，保留合理精度
                if x.fract() == 0.0 {
                    write!(f, "{x:.1}")
                } else {
                    write!(f, "{x}")
                }
            }
            Self::String(s) => write!(f, "{s}"),
            Self::Ref(r) => write!(f, "{r}"),
            Self::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    constant_table: &'static [Value],
    local_tables: Vec<Value>,
    op_stack: Vec<Value>,
    name: String,
    pub(crate) file_name: SmolStr,
    pc: usize, // 指令执行索引(仅当前栈帧)
    codes: &'static [ByteCode],
    native: Option<SmolStr>, // 本地函数栈帧路径
    args: usize,             // 从父栈帧提取的参数个数
}

impl StackFrame {
    pub fn new(
        name: String,
        file_name: SmolStr,
        codes: &'static [ByteCode],
        locals: usize,
        constant_table: &'static [Value],
        native: Option<SmolStr>,
        args: usize,
    ) -> Self {
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

    pub fn get_const(&self, index: usize) -> Option<&Value> {
        self.constant_table.get(index)
    }

    pub const fn get_args(&self) -> usize {
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

    pub const fn next_pc(&mut self) {
        self.pc += 1;
    }

    pub const fn set_next_pc(&mut self, next_pc: usize) {
        self.pc = next_pc;
    }
}

#[derive(Clone)]
pub struct Executor {
    threads: Vec<OpenEXThread>,
    files: Vec<SourceFile>,
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            threads: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn add_thread(&mut self, thread: OpenEXThread) -> &mut OpenEXThread {
        self.threads.push(thread);
        self.threads.last_mut().unwrap()
    }

    pub fn get_path_func(&mut self, path: &str) -> Option<(&'static [Value], IrFunction)> {
        let mut sp = path.split('/');
        let file = sp.next().unwrap();
        let func = sp.next().unwrap();

        for i in 0..self.files.len() {
            if self.files[i].name.split('.').next().unwrap() == file {
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

macro_rules! do_bin_op {
    ($frame:expr, $op_func:expr) => {{
        let v_right = $frame.pop_op_stack().unwrap();
        let v_left = $frame.pop_op_stack().unwrap();
        $frame.push_op_stack($op_func(v_left, v_right)?);
        $frame.next_pc();
    }};
}

pub fn run_executor(
    frame_index: usize,
    executor: &mut Executor,
    thread: &mut OpenEXThread,
) -> Result<(Option<StackFrame>, bool), RuntimeError>
// Option<StackFrame> 不为 None 代表是函数调用请求
// bool 为 true 代表操作栈栈顶作为返回值压入父栈帧
{
    let (mut root_frame, stack_frame) = if frame_index == 0 {
        (None, thread.get_mut_frame(0))
    } else {
        let (left, right) = thread.call_stack.split_at_mut(frame_index);
        // 使用 .get_mut(0) 代替 [0] 更加安全，防止潜在的越界 panic
        (left.get_mut(0), right.get_mut(0).unwrap())
    };

    while let Some(instr) = stack_frame.fetch_current() {
        match instr {
            ByteCode::Push(const_index) => push_stack(stack_frame,*const_index),
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
                let result = root_frame.as_ref().map_or_else(
                    || stack_frame.get_var_table(index),
                    |root| root.get_var_table(index),
                );
                stack_frame.push_op_stack(result.clone());
                stack_frame.next_pc();
            }
            ByteCode::Load(var_index) => load_local(stack_frame,*var_index),
            ByteCode::Store(var_index) => store_local(stack_frame,*var_index),
            ByteCode::Add => do_bin_op!(stack_frame, add_value),
            ByteCode::Sub => do_bin_op!(stack_frame, sub_value),
            ByteCode::Nol => stack_frame.next_pc(),
            ByteCode::GetRef => get_ref(stack_frame),
            ByteCode::Call => return call_func(stack_frame, executor),
            ByteCode::Mul => do_bin_op!(stack_frame, mul_value),
            ByteCode::Div => do_bin_op!(stack_frame, div_value),
            ByteCode::SAdd => self_add_value(stack_frame)?,
            ByteCode::SSub => self_sub_value(stack_frame)?,
            ByteCode::Jump(pc) => jump(stack_frame, *pc),
            ByteCode::JumpTrue(pc) => jump_true(stack_frame, *pc),
            ByteCode::JumpFalse(pc) => jump_false(stack_frame, *pc),
            ByteCode::Big => do_bin_op!(stack_frame, big_value),
            ByteCode::Less => do_bin_op!(stack_frame, less_value),
            ByteCode::Equ => equ_value(stack_frame),
            ByteCode::NotEqu => not_equ_value(stack_frame),
            ByteCode::Not => not_value(stack_frame)?,
            ByteCode::Return => return Ok((None, true)),
            _ => todo!(),
        }
    }
    Ok((None, false))
}
