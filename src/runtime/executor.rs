use crate::compiler::ast::vm_ir::{ByteCode, Value};
use crate::compiler::parser::ParserError;
use crate::library::find_library;
use crate::runtime::vm_operation::{add_value, big_value, div_value, equ_value, get_ref, less_value, mul_value, not_equ_value, not_value, self_add_value, self_sub_value, sub_value};
use crate::runtime::vm_table_opt::{
    call_func, jump, jump_false, jump_true, load_local, push_stack, store_local,
};
use crate::runtime::{MetadataUnit, RuntimeError};
use smol_str::SmolStr;

pub struct StackFrame<'a> {
    pc: usize,
    local: Vec<Value>,
    op_stack: Vec<Value>,
    codes: &'a [ByteCode],
    const_table: &'a [Value],
    is_native: Option<SmolStr>,
    pub name: &'a str,
    pub r_name: &'a str, // 栈帧所属脚本的名称
    args: usize,
}

pub struct Executor<'a> {
    call_stack: Vec<StackFrame<'a>>,
    frame_index: usize,
}

impl<'a> Executor<'a> {
    pub const fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            frame_index: 0,
        }
    }

    pub fn get_top_frame(&mut self) -> Option<&mut StackFrame<'a>> {
        if self.frame_index == 0 {
            None
        } else {
            self.call_stack.get_mut(self.frame_index - 1)
        }
    }

    pub fn push_frame(&mut self, frame: StackFrame<'a>) {
        if self.frame_index >= self.call_stack.len() {
            self.call_stack.push(frame);
        } else {
            self.call_stack[self.frame_index - 1] = frame;
        }
        self.frame_index += 1;
    }
}

impl<'a> StackFrame<'a> {
    pub fn new(
        local_size: usize,
        codes: &'a [ByteCode],
        const_table: &'a [Value],
        name: &'a str,
        r_name: &'a str,
        is_native: Option<SmolStr>,
        args: usize,
    ) -> Self {
        Self {
            pc: 0,
            local: vec![Value::Null; local_size],
            op_stack: Vec::new(),
            r_name,
            codes,
            const_table,
            name,
            is_native,
            args,
        }
    }

    pub const fn get_frame_name(&self) -> &'a str {
        self.name
    }

    pub const fn get_args(&self) -> usize {
        self.args
    }

    pub fn get_op_stack_top(&self) -> Option<&Value> {
        self.op_stack.last()
    }

    pub fn set_local(&mut self, index: usize, value: Value) {
        self.local[index] = value;
    }

    pub fn get_local(&self, index: usize) -> &Value {
        &self.local[index]
    }

    pub fn push_op_stack(&mut self, value: Value) {
        self.op_stack.push(value);
    }

    pub fn pop_op_stack(&mut self) -> Value {
        self.op_stack.pop().unwrap()
    }

    pub const fn next_pc(&mut self) {
        self.pc += 1;
    }

    pub const fn set_next_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    pub fn current_code(&self) -> Option<&ByteCode> {
        if self.pc >= self.codes.len() {
            None
        } else {
            Some(&self.codes[self.pc])
        }
    }

    pub fn get_const(&self, index: usize) -> Option<&Value> {
        if index >= self.const_table.len() {
            None
        } else {
            Some(&self.const_table[index])
        }
    }

    pub const fn is_native(&self) -> Option<&SmolStr> {
        self.is_native.as_ref()
    }
}

pub enum RunState<'a> {
    CallRequest(StackFrame<'a>), // 函数调用请求
    Return,                      // 返回需要将子栈帧栈顶压入父栈帧操作栈
    None,                        // 空操作
}

macro_rules! do_bin_op {
    ($frame:expr, $op_func:expr) => {{
        let v_right = $frame.pop_op_stack();
        let v_left = $frame.pop_op_stack();
        $frame.push_op_stack($op_func(v_left, v_right)?);
        $frame.next_pc();
    }};
}

fn run_code<'a>(
    units: &'a [MetadataUnit<'_>],
    stack_frame: &mut StackFrame,
    mut root_frame: Option<&mut StackFrame>,
) -> Result<RunState<'a>, RuntimeError> {
    while let Some(code) = stack_frame.current_code() {
        match code {
            ByteCode::Push(const_index) => push_stack(stack_frame, *const_index),
            ByteCode::Load(local_index) => load_local(stack_frame, *local_index),
            ByteCode::Store(local_index) => store_local(stack_frame, *local_index),
            ByteCode::GetRef => get_ref(stack_frame),
            ByteCode::Add => do_bin_op!(stack_frame, add_value),
            ByteCode::Sub => do_bin_op!(stack_frame, sub_value),
            ByteCode::Mul => do_bin_op!(stack_frame, mul_value),
            ByteCode::Div => do_bin_op!(stack_frame, div_value),
            ByteCode::Call => return call_func(stack_frame, units),
            ByteCode::Return => return Ok(RunState::Return),
            ByteCode::Jump(pc) => jump(stack_frame, *pc),
            ByteCode::JumpTrue(pc) => jump_true(stack_frame, *pc),
            ByteCode::JumpFalse(pc) => jump_false(stack_frame, *pc),
            ByteCode::Equ => equ_value(stack_frame),
            ByteCode::NotEqu => not_equ_value(stack_frame),
            ByteCode::Not => not_value(stack_frame)?,
            ByteCode::SAdd => self_add_value(stack_frame)?,
            ByteCode::SSub => self_sub_value(stack_frame)?,
            ByteCode::Big => do_bin_op!(stack_frame, big_value),
            ByteCode::Less => do_bin_op!(stack_frame, less_value),
            ByteCode::LoadGlobal(var_index) => {
                let index = *var_index;
                let result = stack_frame.pop_op_stack();
                if let Some(ref mut root) = root_frame {
                    root.set_local(index, result);
                } else {
                    stack_frame.set_local(index, result);
                }
                stack_frame.next_pc();
            }
            ByteCode::StoreGlobal(var_index) => {
                let index = *var_index;
                let result = root_frame.as_ref().map_or_else(
                    || stack_frame.get_local(index),
                    |root| root.get_local(index),
                );
                stack_frame.push_op_stack(result.clone());
                stack_frame.next_pc();
            }
            ByteCode::Nol => stack_frame.next_pc(),
            _ => todo!(),
        }
    }
    Ok(RunState::None)
}

pub fn interpretive(
    codes: &[ByteCode],
    const_table: &[Value],
    name: &str,
    units: &[MetadataUnit],
    globals: usize,
) {
    let mut executor = Executor::new();
    executor.push_frame(StackFrame::new(globals, codes, const_table, name, name,
        None,
        0,
    ));
    let mut failed_status = None;
    loop {
        let size = executor.call_stack.len();
        let (root_frame, stack_frame) = if size > 2 {
            let stack_frame = executor.get_top_frame().unwrap();
            (None, stack_frame)
        } else {
            if size < 1 {
                break;
            }
            let (left, right) = executor.call_stack.split_at_mut(size - 1);
            (left.get_mut(0), right.get_mut(0).unwrap())
        };

        if let Some(path) = stack_frame.is_native() {
            let path = path.clone();
            let mut sp = path.split('/');
            let file = sp.next().unwrap();
            let func = sp.next().unwrap();

            let mut argument = vec![];
            for _i in 0..stack_frame.get_args() {
                argument.push(stack_frame.pop_op_stack());
            }

            if let Ok(lib) = find_library(file, |f| {
                if let Some(lib) = f
                    && let Some(func) = lib.find_func(&SmolStr::new(func))
                {
                    (func.func)(&argument).map_or(Err(ParserError::Empty), Ok)
                } else {
                    Err(ParserError::Empty)
                }
            }) {
                executor.call_stack.last_mut().unwrap().push_op_stack(lib);
                executor.call_stack.pop().unwrap();
                executor.frame_index -= 1;
            } else {
                eprintln!(
                    "RuntimeError: {:?}",
                    RuntimeError::NoSuchFunctionException(path)
                );
                for frame in &mut executor.call_stack {
                    let name = frame.get_frame_name();
                    eprintln!("\t at <{name}>");
                }
                break;
            }
        } else {
            match run_code(units, stack_frame, root_frame) {
                Ok(state) => match state {
                    RunState::CallRequest(frame) => {
                        executor.push_frame(frame);
                        let mut stack_frame = executor.call_stack.pop().unwrap();
                        if stack_frame.get_args() > 0 {
                            let mut parent_frame = executor.call_stack.pop().unwrap();
                            for _i in 0..stack_frame.get_args() {
                                let value = parent_frame.pop_op_stack();
                                stack_frame.push_op_stack(value);
                            }
                            executor.call_stack.push(parent_frame);
                        }
                        executor.call_stack.push(stack_frame);
                    }
                    RunState::Return => {
                        let frame = executor.call_stack.pop().unwrap();
                        executor.frame_index -= 1;
                        if let Some(ret_var) = frame.get_op_stack_top() {
                            executor
                                .call_stack
                                .last_mut()
                                .unwrap()
                                .push_op_stack(ret_var.clone());
                        }
                    }
                    RunState::None => {
                        executor.call_stack.pop();
                        executor.frame_index -= 1;
                    }
                },
                Err(state) => {
                    //TODO 需要做栈帧异常回溯
                    failed_status = Some(state);
                    break;
                }
            }
        }
    }

    if let Some(error) = failed_status {
        eprintln!("RuntimeError: {error:?}");
        for frame in &mut executor.call_stack {
            let name = frame.get_frame_name();
            eprintln!("\t at <{name}>");
        }
    }
}
