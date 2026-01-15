use dashu::float::DBig;
use smol_str::{SmolStr, ToSmolStr, format_smolstr};

use crate::compiler::ast::vm_ir::{ByteCode, Value};
use crate::compiler::parser::ParserError;
use crate::library::find_library;
use crate::runtime::vm_operation::*;
use crate::runtime::vm_table_opt::*;
use crate::runtime::{GlobalStore, MetadataUnit, RuntimeError};

pub struct StackFrame<'a> {
    pc: usize,
    local: Vec<Value>,
    op_stack: Vec<Value>,
    codes: &'a [ByteCode],
    const_table: &'a [Value],
    is_native: Option<SmolStr>,
    pub name: &'a str,
    pub r_name: &'a str, // 栈帧所属脚本的名称
    unit_index: usize,
    args: usize,
    memo_target: Option<(usize, usize)>,
    memo_key: Option<Vec<MemoKey>>,
    sync_lock: Option<(usize, usize)>,
}

pub struct Executor<'a> {
    call_stack: Vec<StackFrame<'a>>,
    frame_index: usize,
}

impl<'a> Default for Executor<'a> {
    fn default() -> Self {
        Self::new()
    }
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
        unit_index: usize,
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
            unit_index,
            args,
            memo_target: None,
            memo_key: None,
            sync_lock: None,
        }
    }

    pub const fn get_frame_name(&self) -> &'a str {
        self.name
    }

    pub const fn get_args(&self) -> usize {
        self.args
    }

    pub const fn get_unit_index(&self) -> usize {
        self.unit_index
    }

    #[must_use]
    pub fn get_op_stack_top(&self) -> Option<&Value> {
        self.op_stack.last()
    }

    pub fn set_local(&mut self, index: usize, value: Value) {
        self.local[index] = value;
    }

    #[must_use]
    pub fn get_local(&self, index: usize) -> &Value {
        &self.local[index]
    }

    pub fn get_local_mut(&mut self, index: usize) -> &mut Value {
        &mut self.local[index]
    }

    pub fn push_op_stack(&mut self, value: Value) {
        self.op_stack.push(value);
    }

    pub fn peek_args(&self, count: usize) -> Option<&[Value]> {
        let len = self.op_stack.len();
        if len < count {
            None
        } else {
            Some(&self.op_stack[len - count..])
        }
    }

    /// # Panics
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

    pub fn set_memo(&mut self, target: (usize, usize), key: Vec<MemoKey>) {
        self.memo_target = Some(target);
        self.memo_key = Some(key);
    }

    pub fn take_memo(&mut self) -> Option<(usize, usize, Vec<MemoKey>)> {
        match (self.memo_target.take(), self.memo_key.take()) {
            (Some(target), Some(key)) => Some((target.0, target.1, key)),
            _ => None,
        }
    }

    pub fn set_sync_lock(&mut self, target: (usize, usize)) {
        self.sync_lock = Some(target);
    }

    pub fn take_sync_lock(&mut self) -> Option<(usize, usize)> {
        self.sync_lock.take()
    }
}

pub enum RunState<'a> {
    CallRequest(StackFrame<'a>), // 函数调用请求
    Return,                      // 返回需要将子栈帧栈顶压入父栈帧操作栈
    Continue,                    // 继续执行当前栈帧
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
    globals: &mut GlobalStore,
    call_cache: &CallCache,
) -> Result<RunState<'a>, RuntimeError> {
    while let Some(code) = stack_frame.current_code() {
        match code {
            ByteCode::Push(const_index) => push_stack(stack_frame, *const_index),
            ByteCode::Pop(len) => {
                for _ in 0..*len {
                    let _ = stack_frame.pop_op_stack();
                }
                stack_frame.next_pc();
            }
            ByteCode::AddLocalImm(index, imm) => {
                let index = *index;
                let imm = *imm;
                let value = stack_frame.get_local_mut(index);
                match value {
                    Value::Int(i) => {
                        *i += imm;
                    }
                    Value::Float(f) => {
                        *f += DBig::from(imm);
                    }
                    auto => {
                        return Err(RuntimeError::TypeException(format_smolstr!(
                            "{auto} to int or float"
                        )));
                    }
                }
                stack_frame.next_pc();
            }
            ByteCode::AddGlobalImm(index, imm) => {
                let index = *index;
                let imm = *imm;
                let unit_index = stack_frame.get_unit_index();
                let Some(value) = globals.get_mut(unit_index, index) else {
                    return Err(RuntimeError::VMError);
                };
                match value {
                    Value::Int(i) => {
                        *i += imm;
                    }
                    Value::Float(f) => {
                        *f += DBig::from(imm);
                    }
                    auto => {
                        return Err(RuntimeError::TypeException(format_smolstr!(
                            "{auto} to int or float"
                        )));
                    }
                }
                stack_frame.next_pc();
            }
            ByteCode::Load(local_index) => load_local(stack_frame, *local_index),
            ByteCode::Store(local_index) => store_local(stack_frame, *local_index),
            ByteCode::GetRef => get_ref(stack_frame),
            ByteCode::Add => do_bin_op!(stack_frame, add_value),
            ByteCode::Sub => do_bin_op!(stack_frame, sub_value),
            ByteCode::Mul => do_bin_op!(stack_frame, mul_value),
            ByteCode::Div => do_bin_op!(stack_frame, div_value),
            ByteCode::Rmd => do_bin_op!(stack_frame, rmd_value),
            ByteCode::Call => match call_func(stack_frame, units, call_cache) {
                Ok(RunState::Continue) => continue,
                Ok(state) => return Ok(state),
                Err(err) => return Err(err),
            },
            ByteCode::Return => return Ok(RunState::Return),
            ByteCode::Jump(pc) => jump(stack_frame, *pc),
            ByteCode::JumpTrue(pc) => jump_true(stack_frame, *pc),
            ByteCode::JumpFalse(pc) => jump_false(stack_frame, *pc),
            ByteCode::Equ => equ_value(stack_frame),
            ByteCode::NotEqu => not_equ_value(stack_frame),
            ByteCode::Not => not_value(stack_frame)?,
            ByteCode::Neg => neg_value(stack_frame)?,
            ByteCode::SAdd => self_add_value(stack_frame)?,
            ByteCode::SSub => self_sub_value(stack_frame)?,
            ByteCode::And => and_value(stack_frame)?,
            ByteCode::Or => or_value(stack_frame)?,
            ByteCode::Big => do_bin_op!(stack_frame, big_value),
            ByteCode::Less => do_bin_op!(stack_frame, less_value),
            ByteCode::LesEqu => do_bin_op!(stack_frame, less_equ_value),
            ByteCode::BLeft => bit_left_value(stack_frame)?,
            ByteCode::BRight => bit_right_value(stack_frame)?,
            ByteCode::BitAnd => bit_and_value(stack_frame)?,
            ByteCode::BitOr => bit_or_value(stack_frame)?,
            ByteCode::BitXor => bit_xor_value(stack_frame)?,
            ByteCode::LoadGlobal(var_index) => {
                let index = *var_index;
                let result = stack_frame.pop_op_stack();
                let unit_index = stack_frame.get_unit_index();
                let Some(slot) = globals.get_mut(unit_index, index) else {
                    return Err(RuntimeError::VMError);
                };
                *slot = result;
                stack_frame.next_pc();
            }
            ByteCode::StoreGlobal(var_index) => {
                let index = *var_index;
                let unit_index = stack_frame.get_unit_index();
                let Some(value) = globals.get(unit_index, index) else {
                    return Err(RuntimeError::VMError);
                };
                stack_frame.push_op_stack(value.clone());
                stack_frame.next_pc();
            }
            ByteCode::LoadArrayGlobal(var_index, len) => {
                let len_s = *len;
                let index = *var_index;
                let mut elements: Vec<Value> = Vec::new();
                for _ in 0..len_s {
                    elements.push(stack_frame.pop_op_stack());
                }

                let reversed_values: Vec<Value> = elements.into_iter().rev().collect();

                let result = Value::Array(len_s, reversed_values);
                let unit_index = stack_frame.get_unit_index();
                let Some(slot) = globals.get_mut(unit_index, index) else {
                    return Err(RuntimeError::VMError);
                };
                *slot = result;
                stack_frame.next_pc();
            }
            ByteCode::SetArray(var_index) => set_index_array(stack_frame, *var_index)?,
            ByteCode::SetArrayGlobal(var_index) => {
                let index = *var_index;
                let arr_index = stack_frame.pop_op_stack();
                let value = stack_frame.pop_op_stack();
                let unit_index = stack_frame.get_unit_index();
                let Some(result) = globals.get_mut(unit_index, index) else {
                    return Err(RuntimeError::VMError);
                };
                if let Value::Array(len, elements) = result
                    && let Value::Int(a_index) = arr_index
                {
                    let usize_index = usize::try_from(a_index).unwrap();
                    if usize_index >= *len {
                        return Err(RuntimeError::IndexOutOfBounds(
                            format_args!("Index {a_index} out of bounds for length {len}")
                                .to_smolstr(),
                        ));
                    }
                    elements[usize_index] = value;
                    stack_frame.next_pc();
                } else {
                    return Err(RuntimeError::TypeException(
                        "cannot set unknown type for array.".to_smolstr(),
                    ));
                }
            }
            ByteCode::LoadArray(var_index, len) => load_array_local(stack_frame, *len, *var_index),
            ByteCode::GetIndex => get_index_array(stack_frame)?,
            ByteCode::GetIndexLocal(var_index) => get_index_local(stack_frame, *var_index)?,
            ByteCode::Nol | ByteCode::Pos => stack_frame.next_pc(),
            _ => todo!(),
        }
    }
    Ok(RunState::None)
}

fn print_and_return(executor: &Executor, failed_status: Option<RuntimeError>) -> Value {
    if let Some(error) = failed_status {
        eprintln!("RuntimeError: {error:?}");
        for frame in &executor.call_stack {
            let name = frame.get_frame_name();
            eprintln!("\t at <{name}>");
        }
    }
    Value::Null
}

fn print_error(executor: &Executor, path: SmolStr) {
    eprintln!(
        "RuntimeError: {:?}",
        RuntimeError::NoSuchFunctionException(path)
    );
    for frame in &executor.call_stack {
        let name = frame.get_frame_name();
        eprintln!("\t at <{name}>");
    }
}

pub fn call_function(
    codes: &[ByteCode],
    const_table: &[Value],
    name: &str,
    units: &[MetadataUnit],
    unit_index: usize,
    local_size: usize,
    globals: &mut GlobalStore,
    arguments: Vec<Value>,
) -> Value {
    let mut executor = Executor::new();
    let mut call_cache = CallCache::new(units);
    executor.push_frame(StackFrame::new(
        unit_index,
        local_size,
        codes,
        const_table,
        name,
        name,
        None,
        0,
    ));
    let mut failed_status = None;
    for arg in arguments {
        executor.get_top_frame().unwrap().push_op_stack(arg);
    }

    loop {
        if executor.call_stack.is_empty() {
            break;
        }
        let stack_frame = executor.get_top_frame().unwrap();

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
                let mut frame = executor.call_stack.pop().unwrap();
                if let Some((unit_index, func_index)) = frame.take_sync_lock() {
                    call_cache.unlock_sync(unit_index, func_index);
                }
                executor.call_stack.last_mut().unwrap().push_op_stack(lib);
                executor.frame_index -= 1;
            } else {
                print_error(&executor, path);
                break;
            }
        } else {
            match run_code(units, stack_frame, globals, &call_cache) {
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
                        let mut frame = executor.call_stack.pop().unwrap();
                        executor.frame_index -= 1;
                        if let Some((unit_index, func_index)) = frame.take_sync_lock() {
                            call_cache.unlock_sync(unit_index, func_index);
                        }
                        if let Some(ret_var) = frame.get_op_stack_top().cloned() {
                            if let Some((unit_index, func_index, key)) = frame.take_memo() {
                                call_cache.store_memo(unit_index, func_index, key, ret_var.clone());
                            }
                            if executor.call_stack.is_empty() {
                                return ret_var;
                            }
                            executor
                                .call_stack
                                .last_mut()
                                .unwrap()
                                .push_op_stack(ret_var);
                        }
                    }
                    RunState::Continue => {}
                    RunState::None => {
                        let mut frame = executor.call_stack.pop().unwrap();
                        if let Some((unit_index, func_index)) = frame.take_sync_lock() {
                            call_cache.unlock_sync(unit_index, func_index);
                        }
                        executor.frame_index -= 1;
                    }
                },
                Err(state) => {
                    //TODO 需要做栈帧异常回溯
                    for frame in executor.call_stack.iter_mut() {
                        if let Some((unit_index, func_index)) = frame.take_sync_lock() {
                            call_cache.unlock_sync(unit_index, func_index);
                        }
                    }
                    failed_status = Some(state);
                    break;
                }
            }
        }
    }

    print_and_return(&executor, failed_status)
}

pub fn interpretive(
    codes: &[ByteCode],
    const_table: &[Value],
    name: &str,
    units: &[MetadataUnit],
    unit_index: usize,
    local_size: usize,
    globals: &mut GlobalStore,
) {
    call_function(
        codes,
        const_table,
        name,
        units,
        unit_index,
        local_size,
        globals,
        Vec::new(),
    );
}
