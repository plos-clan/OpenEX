use smol_str::{SmolStr, ToSmolStr, format_smolstr};
use std::collections::{HashMap, HashSet};

use crate::compiler::ast::vm_ir::{ByteCode, Value};
use crate::runtime::context::SyncTable;
use crate::runtime::executor::{RunState, StackFrame};
use crate::runtime::{MetadataUnit, RuntimeError};

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum MemoKey {
    Int(i64),
    Bool(bool),
    String(SmolStr),
    Ref(SmolStr),
    Null,
}

impl MemoKey {
    fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Int(v) => Some(MemoKey::Int(*v)),
            Value::Bool(v) => Some(MemoKey::Bool(*v)),
            Value::String(v) => Some(MemoKey::String(v.clone())),
            Value::Ref(v) => Some(MemoKey::Ref(v.clone())),
            Value::Null => Some(MemoKey::Null),
            _ => None,
        }
    }
}

pub struct CallCache {
    map: HashMap<SmolStr, (usize, usize)>,
    memoizable: HashSet<(usize, usize)>,
    memo: HashMap<(usize, usize), HashMap<Vec<MemoKey>, Value>>,
}

impl CallCache {
    pub fn new(units: &[MetadataUnit]) -> Self {
        let mut map = HashMap::new();
        let mut memoizable = HashSet::new();
        let mut memo = HashMap::new();
        for (unit_index, unit) in units.iter().enumerate() {
            for (func_index, func) in unit.methods.iter().enumerate() {
                let self_path = format_smolstr!("{}/{}", unit.names, func.name);
                map.insert(self_path.clone(), (unit_index, func_index));
                if is_pure_self_recursive(unit, func, &self_path) {
                    let idx = (unit_index, func_index);
                    memoizable.insert(idx);
                    memo.insert(idx, HashMap::new());
                }
            }
        }
        Self {
            map,
            memoizable,
            memo,
        }
    }

    pub fn resolve(&self, path: &SmolStr) -> Option<(usize, usize)> {
        self.map.get(path).copied()
    }

    pub fn is_memoizable(&self, unit_index: usize, func_index: usize) -> bool {
        self.memoizable.contains(&(unit_index, func_index))
    }

    pub fn make_key(values: &[Value]) -> Option<Vec<MemoKey>> {
        let mut key = Vec::with_capacity(values.len());
        for value in values {
            key.push(MemoKey::from_value(value)?);
        }
        Some(key)
    }

    pub fn get_memo(&self, unit_index: usize, func_index: usize, key: &[MemoKey]) -> Option<Value> {
        self.memo
            .get(&(unit_index, func_index))
            .and_then(|map| map.get(key).cloned())
    }

    pub fn store_memo(
        &mut self,
        unit_index: usize,
        func_index: usize,
        key: Vec<MemoKey>,
        value: Value,
    ) {
        if let Some(map) = self.memo.get_mut(&(unit_index, func_index)) {
            map.entry(key).or_insert(value);
        }
    }
}

fn is_pure_self_recursive(
    unit: &MetadataUnit,
    func: &crate::runtime::MethodInfo,
    self_path: &SmolStr,
) -> bool {
    if func.is_native {
        return false;
    }
    let codes = func.get_codes();
    for (idx, code) in codes.iter().enumerate() {
        match code {
            ByteCode::StoreGlobal(_)
            | ByteCode::LoadGlobal(_)
            | ByteCode::LoadArrayGlobal(_, _)
            | ByteCode::SetArrayGlobal(_)
            | ByteCode::AddGlobalImm(_, _) => {
                return false;
            }
            ByteCode::Call => {
                if idx == 0 {
                    return false;
                }
                match codes.get(idx - 1) {
                    Some(ByteCode::Push(const_index)) => {
                        match unit.constant_table.get(*const_index) {
                            Some(Value::Ref(path)) if path == self_path => {}
                            _ => return false,
                        }
                    }
                    _ => return false,
                }
            }
            _ => {}
        }
    }
    true
}

pub fn push_stack(stack_frame: &mut StackFrame, index: usize) {
    let Some(value_ref) = stack_frame.get_const(index) else {
        unimplemented!()
    };
    let final_value = if let Value::Ref(path) = value_ref {
        if path.as_str() == "this" {
            let file_base = stack_frame.r_name;
            Value::Ref(file_base.into())
        } else {
            value_ref.clone()
        }
    } else {
        value_ref.clone()
    };
    stack_frame.push_op_stack(final_value);
    stack_frame.next_pc();
}

pub fn load_local(stack_frame: &mut StackFrame, index: usize) {
    let result = stack_frame.pop_op_stack();
    stack_frame.set_local(index, result);
    stack_frame.next_pc();
}

pub fn store_local(stack_frame: &mut StackFrame, index: usize) {
    let value = stack_frame.get_local(index);
    stack_frame.push_op_stack(value.clone());
    stack_frame.next_pc();
}

pub fn jump_true(stack_frame: &mut StackFrame, jpc: usize) {
    let top = stack_frame.pop_op_stack();
    let Value::Bool(value) = top else {
        unreachable!()
    };
    if value {
        stack_frame.set_next_pc(jpc);
    } else {
        stack_frame.next_pc();
    }
}

pub fn jump_false(stack_frame: &mut StackFrame, jpc: usize) {
    let top = stack_frame.pop_op_stack();
    let Value::Bool(value) = top else {
        unreachable!()
    };
    if value {
        stack_frame.next_pc();
    } else {
        stack_frame.set_next_pc(jpc);
    }
}

pub const fn jump(stack_frame: &mut StackFrame, jpc: usize) {
    stack_frame.set_next_pc(jpc);
}

pub fn call_func<'a>(
    stack_frame: &mut StackFrame,
    units: &'a [MetadataUnit],
    call_cache: &CallCache,
    sync_table: &SyncTable,
) -> Result<RunState<'a>, RuntimeError> {
    let result = stack_frame.pop_op_stack();

    let Value::Ref(path) = result else {
        return Err(RuntimeError::VMError);
    };

    let Some((unit_index, func_index)) = call_cache.resolve(&path) else {
        return Err(RuntimeError::NoSuchFunctionException(path));
    };

    let unit = &units[unit_index];
    let func = &unit.methods[func_index];
    let codes = func.get_codes();
    let sync_locked = sync_table.lock_if_sync(unit_index, func_index);
    if call_cache.is_memoizable(unit_index, func_index)
        && let Some(args) = stack_frame.peek_args(func.args)
        && let Some(key) = CallCache::make_key(args)
    {
        if let Some(value) = call_cache.get_memo(unit_index, func_index, &key) {
            for _ in 0..func.args {
                let _ = stack_frame.pop_op_stack();
            }
            stack_frame.push_op_stack(value);
            stack_frame.next_pc();
            if sync_locked {
                sync_table.unlock(unit_index, func_index);
            }
            return Ok(RunState::Continue);
        }
        let native = if func.is_native { Some(path) } else { None };
        let mut frame = StackFrame::new(
            unit_index,
            func.locals,
            codes,
            unit.constant_table,
            func.name.as_str(),
            func.r_name.as_str(),
            native,
            func.args,
        );
        frame.set_memo((unit_index, func_index), key);
        if sync_locked {
            frame.set_sync_lock((unit_index, func_index));
        }
        stack_frame.next_pc();
        return Ok(RunState::CallRequest(frame));
    }
    stack_frame.next_pc();
    let native = if func.is_native { Some(path) } else { None };
    let mut frame = StackFrame::new(
        unit_index,
        func.locals,
        codes,
        unit.constant_table,
        func.name.as_str(),
        func.r_name.as_str(),
        native,
        func.args,
    );
    if sync_locked {
        frame.set_sync_lock((unit_index, func_index));
    }
    Ok(RunState::CallRequest(frame))
}

pub fn load_array_local(stack_frame: &mut StackFrame, len: usize, index: usize) {
    let mut elements: Vec<Value> = Vec::new();
    for _ in 0..len {
        elements.push(stack_frame.pop_op_stack());
    }
    let reversed_values: Vec<Value> = elements.into_iter().rev().collect();
    let result = Value::Array(len, reversed_values);
    stack_frame.set_local(index, result);
    stack_frame.next_pc();
}

pub fn set_index_array(stack_frame: &mut StackFrame, index: usize) -> Result<(), RuntimeError> {
    let arr_index = stack_frame.pop_op_stack();
    let value = stack_frame.pop_op_stack();
    let result = stack_frame.get_local_mut(index);

    if let Value::Array(len, elements) = result
        && let Value::Int(a_index) = arr_index
    {
        let usize_index = usize::try_from(a_index).unwrap();
        if usize_index >= *len {
            return Err(RuntimeError::IndexOutOfBounds(
                format_args!("Index {a_index} out of bounds for length {len}").to_smolstr(),
            ));
        }
        elements[usize_index] = value;
        stack_frame.next_pc();
        Ok(())
    } else {
        Err(RuntimeError::TypeException(
            "cannot set unknown type for array.".to_smolstr(),
        ))
    }
}

pub fn get_index_array(stack_frame: &mut StackFrame) -> Result<(), RuntimeError> {
    let index = stack_frame.pop_op_stack();
    let array = stack_frame.pop_op_stack();
    if let Value::Int(index) = index
        && let Value::Array { 0: len, 1: element } = array
    {
        if usize::try_from(index).unwrap() >= len {
            return Err(RuntimeError::IndexOutOfBounds(
                format_args!("Index {index} out of bounds for length {len}").to_smolstr(),
            ));
        }
        let result = element
            .get(usize::try_from(index).unwrap())
            .unwrap()
            .clone();
        stack_frame.push_op_stack(result);
        stack_frame.next_pc();
        Ok(())
    } else {
        Err(RuntimeError::TypeException(
            "cannot get_index unknown type.".to_smolstr(),
        ))
    }
}

pub fn get_index_local(stack_frame: &mut StackFrame, index: usize) -> Result<(), RuntimeError> {
    let arr_index = stack_frame.pop_op_stack();
    let array = stack_frame.get_local(index);
    if let Value::Int(arr_index) = arr_index
        && let Value::Array { 0: len, 1: element } = array
    {
        let usize_index = usize::try_from(arr_index).unwrap();
        if usize_index >= *len {
            return Err(RuntimeError::IndexOutOfBounds(
                format_args!("Index {arr_index} out of bounds for length {len}").to_smolstr(),
            ));
        }
        let result = element.get(usize_index).unwrap().clone();
        stack_frame.push_op_stack(result);
        stack_frame.next_pc();
        Ok(())
    } else {
        Err(RuntimeError::TypeException(
            "cannot get_index unknown type.".to_smolstr(),
        ))
    }
}
