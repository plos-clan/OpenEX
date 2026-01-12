use crate::compiler::ast::ExprOp;
use crate::compiler::ast::ssa_ir::Operand::{ImmBool, ImmFlot, ImmNum, ImmStr, Library, Reference};
use crate::compiler::ast::ssa_ir::{Code, LocalAddr, OpCode, OpCodeTable, Operand};
use dashu::float::{Context, DBig};
use slotmap::DefaultKey;
use smol_str::{SmolStr, SmolStrBuilder};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub fn unary_optimizer(op: ExprOp, operand: &Operand) -> Option<Operand> {
    match op {
        ExprOp::Not => {
            if let ImmBool(b) = operand {
                Some(ImmBool(!b))
            } else {
                None
            }
        }
        ExprOp::SAdd => {
            if let ImmNum(num) = operand {
                Some(ImmNum(num + 1))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(flot + DBig::from(1)))
            } else {
                None
            }
        }
        ExprOp::SSub => {
            if let ImmNum(num) = operand {
                Some(ImmNum(num - 1))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(flot - DBig::from(1)))
            } else {
                None
            }
        }
        ExprOp::Neg => {
            if let ImmNum(num) = operand {
                Some(ImmNum(-num))
            } else if let ImmFlot(flot) = operand {
                Some(ImmFlot(-flot))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn expr_optimizer(left: &Operand, right: &Operand, op: ExprOp) -> Option<Operand> {
    match (left, right, op) {
        (ImmNum(a), ImmNum(b), ExprOp::Add) => Some(ImmNum(a + b)),
        (ImmNum(a), ImmNum(b), ExprOp::Sub) => Some(ImmNum(a - b)),
        (ImmNum(a), ImmNum(b), ExprOp::Mul) => Some(ImmNum(a * b)),
        (ImmNum(a), ImmNum(b), ExprOp::Div) => Some(ImmNum(a / b)),
        (ImmNum(a), ImmNum(b), ExprOp::Rmd) => Some(ImmNum(a % b)),

        (ImmNum(a), ImmFlot(b), ExprOp::Add) => Some(ImmFlot(DBig::from(*a) + b)),
        (ImmNum(a), ImmFlot(b), ExprOp::Sub) => Some(ImmFlot(DBig::from(*a) - b)),
        (ImmNum(a), ImmFlot(b), ExprOp::Mul) => Some(ImmFlot(DBig::from(*a) * b)),
        (ImmNum(a), ImmFlot(b), ExprOp::Div) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.div(DBig::from(*a).repr(), b.repr()).value(),
            ))
        }
        (ImmNum(a), ImmFlot(b), ExprOp::Rmd) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.rem(DBig::from(*a).repr(), b.repr()).value(),
            ))
        }

        (ImmFlot(a), ImmNum(b), ExprOp::Add) => Some(ImmFlot(a + DBig::from(*b))),
        (ImmFlot(a), ImmNum(b), ExprOp::Sub) => Some(ImmFlot(a - DBig::from(*b))),
        (ImmFlot(a), ImmNum(b), ExprOp::Mul) => Some(ImmFlot(a * DBig::from(*b))),
        (ImmFlot(a), ImmNum(b), ExprOp::Div) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.div(a.repr(), DBig::from(*b).repr()).value(),
            ))
        }
        (ImmFlot(a), ImmNum(b), ExprOp::Rmd) => {
            let context = Context::new(30);
            Some(ImmFlot(
                context.rem(a.repr(), DBig::from(*b).repr()).value(),
            ))
        }
        // 位运算
        (ImmNum(a), ImmNum(b), ExprOp::BitAnd) => Some(ImmNum(a & b)),
        (ImmNum(a), ImmNum(b), ExprOp::BitOr) => Some(ImmNum(a | b)),
        (ImmNum(a), ImmNum(b), ExprOp::BitXor) => Some(ImmNum(a ^ b)),
        (ImmNum(a), ImmNum(b), ExprOp::BLeft) => Some(ImmNum(a << b)),
        (ImmNum(a), ImmNum(b), ExprOp::BRight) => Some(ImmNum(a >> b)),

        // 比较
        (ImmNum(a), ImmNum(b), ExprOp::Big) => Some(ImmBool(a > b)),
        (ImmNum(a), ImmNum(b), ExprOp::Less) => Some(ImmBool(a < b)),
        (ImmNum(a), ImmNum(b), ExprOp::BigEqu) => Some(ImmBool(a >= b)),
        (ImmNum(a), ImmNum(b), ExprOp::LesEqu) => Some(ImmBool(a <= b)),
        (ImmNum(a), ImmNum(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmNum(a), ImmNum(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmFlot(a), ImmFlot(b), ExprOp::Add) => Some(ImmFlot(a + b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Sub) => Some(ImmFlot(a - b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Mul) => Some(ImmFlot(a * b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Div) => Some(ImmFlot(a / b)),

        // 浮点比较
        (ImmFlot(a), ImmFlot(b), ExprOp::Big) => Some(ImmBool(a > b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Less) => Some(ImmBool(a < b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::BigEqu) => Some(ImmBool(a >= b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::LesEqu) => Some(ImmBool(a <= b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmFlot(a), ImmFlot(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),
        (ImmBool(a), ImmBool(b), ExprOp::And) => Some(ImmBool(*a && *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Or) => Some(ImmBool(*a || *b)),
        (ImmBool(a), ImmBool(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmBool(a), ImmBool(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (ImmStr(a), ImmStr(b), ExprOp::Equ) => Some(ImmBool(a == b)),
        (ImmStr(a), ImmStr(b), ExprOp::NotEqu) => Some(ImmBool(a != b)),

        (Reference(str) | Library(str), Reference(str1) | Library(str1), ExprOp::Ref) => {
            let mut ref_build = SmolStrBuilder::new();
            ref_build.push_str(str1.as_str());
            ref_build.push('/');
            ref_build.push_str(str.as_str());
            Some(Reference(ref_build.finish()))
        }
        // 赋值 / 复合赋值 / 引用 / 下标 —— 不做常量折叠
        _ => None,
    }
}

fn operand_as_const(operand: &Operand) -> Option<Operand> {
    match operand {
        ImmBool(_)
        | ImmNum(_)
        | ImmFlot(_)
        | ImmStr(_)
        | Operand::Null
        | Reference(_)
        | Library(_)
        | Operand::This => Some(operand.clone()),
        _ => None,
    }
}

fn opcode_to_unary_expr(op: &OpCode) -> Option<ExprOp> {
    match op {
        OpCode::Not(_) => Some(ExprOp::Not),
        OpCode::Neg(_) => Some(ExprOp::Neg),
        OpCode::Pos(_) => Some(ExprOp::Pos),
        OpCode::SAdd(_) => Some(ExprOp::SAdd),
        OpCode::SSub(_) => Some(ExprOp::SSub),
        _ => None,
    }
}

fn opcode_to_binary_expr(op: &OpCode) -> Option<ExprOp> {
    match op {
        OpCode::Add(_) => Some(ExprOp::Add),
        OpCode::Sub(_) => Some(ExprOp::Sub),
        OpCode::Mul(_) => Some(ExprOp::Mul),
        OpCode::Div(_) => Some(ExprOp::Div),
        OpCode::Rmd(_) => Some(ExprOp::Rmd),
        OpCode::And(_) => Some(ExprOp::And),
        OpCode::Or(_) => Some(ExprOp::Or),
        OpCode::Equ(_) => Some(ExprOp::Equ),
        OpCode::NotEqu(_) => Some(ExprOp::NotEqu),
        OpCode::BigEqu(_) => Some(ExprOp::BigEqu),
        OpCode::LesEqu(_) => Some(ExprOp::LesEqu),
        OpCode::Big(_) => Some(ExprOp::Big),
        OpCode::Less(_) => Some(ExprOp::Less),
        OpCode::BitAnd(_) => Some(ExprOp::BitAnd),
        OpCode::BitOr(_) => Some(ExprOp::BitOr),
        OpCode::BitXor(_) => Some(ExprOp::BitXor),
        OpCode::BLeft(_) => Some(ExprOp::BLeft),
        OpCode::BRight(_) => Some(ExprOp::BRight),
        OpCode::Ref(_) => Some(ExprOp::Ref),
        OpCode::Store(_) => Some(ExprOp::Store),
        OpCode::AddS(_) => Some(ExprOp::AddS),
        OpCode::SubS(_) => Some(ExprOp::SubS),
        OpCode::MulS(_) => Some(ExprOp::MulS),
        OpCode::DivS(_) => Some(ExprOp::DivS),
        OpCode::RmdS(_) => Some(ExprOp::RmdS),
        OpCode::BAndS(_) => Some(ExprOp::BAndS),
        OpCode::BOrS(_) => Some(ExprOp::BOrS),
        OpCode::BXorS(_) => Some(ExprOp::BXorS),
        _ => None,
    }
}

fn replace_with_push(op: &mut OpCode, operand: Operand) {
    let id = op.get_id();
    *op = OpCode::Push(Some(id), operand);
}

fn stack_pop(stack: &mut Vec<Option<Operand>>) -> Option<Operand> {
    stack.pop().unwrap_or(None)
}

fn stack_pop_n(stack: &mut Vec<Option<Operand>>, count: usize) {
    for _ in 0..count {
        let _ = stack_pop(stack);
    }
}

fn stack_push_operand(stack: &mut Vec<Option<Operand>>, operand: &Operand) {
    if let Some(constant) = operand_as_const(operand) {
        stack.push(Some(constant));
    } else {
        stack.push(None);
    }
}

fn eval_unary(op: &OpCode, value: Option<Operand>) -> Option<Operand> {
    let operand = value?;
    let expr_op = opcode_to_unary_expr(op)?;
    unary_optimizer(expr_op, &operand)
}

fn eval_binary(op: &OpCode, left: Option<Operand>, right: Option<Operand>) -> Option<Operand> {
    let left = left?;
    let right = right?;
    let expr_op = opcode_to_binary_expr(op)?;
    expr_optimizer(&left, &right, expr_op)
}

fn merge_env(
    left: &HashMap<DefaultKey, Option<Operand>>,
    right: &HashMap<DefaultKey, Option<Operand>>,
) -> HashMap<DefaultKey, Option<Operand>> {
    let mut out = HashMap::new();
    for key in left.keys().chain(right.keys()) {
        let l = left.get(key).cloned().unwrap_or(None);
        let r = right.get(key).cloned().unwrap_or(None);
        if let (Some(lv), Some(rv)) = (l, r) {
            if lv == rv {
                out.insert(*key, Some(lv));
            } else {
                out.insert(*key, None);
            }
        } else {
            out.insert(*key, None);
        }
    }
    out
}

fn analyze_block(
    table: &OpCodeTable,
    order: &[LocalAddr],
    start: usize,
    end: usize,
    env_in: &HashMap<DefaultKey, Option<Operand>>,
    arity_map: &HashMap<SmolStr, usize>,
) -> HashMap<DefaultKey, Option<Operand>> {
    let mut env = env_in.clone();
    let mut stack: Vec<Option<Operand>> = Vec::new();

    for idx in start..=end {
        let addr = order[idx];
        let op = table.opcodes.get(&addr).unwrap();
        match op {
            OpCode::Push(_, imm) => {
                stack_push_operand(&mut stack, imm);
            }
            OpCode::Pop(_, len) => {
                stack_pop_n(&mut stack, *len);
            }
            OpCode::StoreLocal(_, key, _) => {
                if let Some(Some(constant)) = env.get(key) {
                    stack.push(Some(constant.clone()));
                } else {
                    stack.push(None);
                }
            }
            OpCode::LoadLocal(_, key, _) => {
                let value = stack_pop(&mut stack);
                env.insert(*key, value);
            }
            OpCode::StoreGlobal(_, _, _) => {
                stack.push(None);
            }
            OpCode::LoadGlobal(_, _, _) => {
                let _ = stack_pop(&mut stack);
            }
            OpCode::LoadArrayLocal(_, key, len) => {
                stack_pop_n(&mut stack, *len);
                env.insert(*key, None);
            }
            OpCode::LoadArrayGlobal(_, _, len) => {
                stack_pop_n(&mut stack, *len);
            }
            OpCode::SetArrayLocal(_, key) => {
                stack_pop_n(&mut stack, 2);
                env.insert(*key, None);
            }
            OpCode::SetArrayGlobal(_, _) => {
                stack_pop_n(&mut stack, 2);
            }
            OpCode::AIndex(_) => {
                stack_pop_n(&mut stack, 2);
                stack.push(None);
            }
            OpCode::Call(_, name) => {
                if let Some(arity) = arity_map.get(name) {
                    stack_pop_n(&mut stack, *arity + 1);
                } else {
                    stack.clear();
                }
                stack.push(None);
            }
            OpCode::JumpTrue(_, _, _) | OpCode::JumpFalse(_, _, _) => {
                let _ = stack_pop(&mut stack);
            }
            OpCode::Jump(_, _) | OpCode::LazyJump(_, _, _) | OpCode::Return(_) => {}
            OpCode::Nop(_) => {}
            OpCode::Not(_)
            | OpCode::Neg(_)
            | OpCode::Pos(_)
            | OpCode::SAdd(_)
            | OpCode::SSub(_) => {
                let value = stack_pop(&mut stack);
                let folded = eval_unary(op, value);
                stack.push(folded);
            }
            OpCode::Add(_)
            | OpCode::Sub(_)
            | OpCode::Mul(_)
            | OpCode::Div(_)
            | OpCode::Rmd(_)
            | OpCode::And(_)
            | OpCode::Or(_)
            | OpCode::Equ(_)
            | OpCode::NotEqu(_)
            | OpCode::BigEqu(_)
            | OpCode::LesEqu(_)
            | OpCode::Big(_)
            | OpCode::Less(_)
            | OpCode::BitAnd(_)
            | OpCode::BitOr(_)
            | OpCode::BitXor(_)
            | OpCode::BLeft(_)
            | OpCode::BRight(_)
            | OpCode::Ref(_)
            | OpCode::Store(_)
            | OpCode::AddS(_)
            | OpCode::SubS(_)
            | OpCode::MulS(_)
            | OpCode::DivS(_)
            | OpCode::RmdS(_)
            | OpCode::BAndS(_)
            | OpCode::BOrS(_)
            | OpCode::BXorS(_) => {
                let right = stack_pop(&mut stack);
                let left = stack_pop(&mut stack);
                let folded = eval_binary(op, left, right);
                stack.push(folded);
            }
        }
    }
    env
}

fn rewrite_block(
    table: &mut OpCodeTable,
    order: &[LocalAddr],
    start: usize,
    end: usize,
    env_in: &HashMap<DefaultKey, Option<Operand>>,
    arity_map: &HashMap<SmolStr, usize>,
) {
    let mut env = env_in.clone();
    let mut stack: Vec<Option<Operand>> = Vec::new();

    for idx in start..=end {
        let addr = order[idx];
        let op = table.opcodes.get_mut(&addr).unwrap();
        match op {
            OpCode::Push(_, imm) => {
                stack_push_operand(&mut stack, imm);
            }
            OpCode::Pop(_, len) => {
                stack_pop_n(&mut stack, *len);
            }
            OpCode::StoreLocal(_, key, _) => {
                if let Some(Some(constant)) = env.get(key) {
                    replace_with_push(op, constant.clone());
                    stack.push(Some(constant.clone()));
                } else {
                    stack.push(None);
                }
            }
            OpCode::LoadLocal(_, key, _) => {
                let value = stack_pop(&mut stack);
                env.insert(*key, value);
            }
            OpCode::StoreGlobal(_, _, _) => {
                stack.push(None);
            }
            OpCode::LoadGlobal(_, _, _) => {
                let _ = stack_pop(&mut stack);
            }
            OpCode::LoadArrayLocal(_, key, len) => {
                stack_pop_n(&mut stack, *len);
                env.insert(*key, None);
            }
            OpCode::LoadArrayGlobal(_, _, len) => {
                stack_pop_n(&mut stack, *len);
            }
            OpCode::SetArrayLocal(_, key) => {
                stack_pop_n(&mut stack, 2);
                env.insert(*key, None);
            }
            OpCode::SetArrayGlobal(_, _) => {
                stack_pop_n(&mut stack, 2);
            }
            OpCode::AIndex(_) => {
                stack_pop_n(&mut stack, 2);
                stack.push(None);
            }
            OpCode::Call(_, name) => {
                if let Some(arity) = arity_map.get(name) {
                    stack_pop_n(&mut stack, *arity + 1);
                } else {
                    stack.clear();
                }
                stack.push(None);
            }
            OpCode::JumpTrue(_, _, _) | OpCode::JumpFalse(_, _, _) => {
                let _ = stack_pop(&mut stack);
            }
            OpCode::Jump(_, _) | OpCode::LazyJump(_, _, _) | OpCode::Return(_) => {}
            OpCode::Nop(_) => {}
            OpCode::Not(_)
            | OpCode::Neg(_)
            | OpCode::Pos(_)
            | OpCode::SAdd(_)
            | OpCode::SSub(_) => {
                let value = stack_pop(&mut stack);
                let folded = eval_unary(op, value);
                stack.push(folded);
            }
            OpCode::Add(_)
            | OpCode::Sub(_)
            | OpCode::Mul(_)
            | OpCode::Div(_)
            | OpCode::Rmd(_)
            | OpCode::And(_)
            | OpCode::Or(_)
            | OpCode::Equ(_)
            | OpCode::NotEqu(_)
            | OpCode::BigEqu(_)
            | OpCode::LesEqu(_)
            | OpCode::Big(_)
            | OpCode::Less(_)
            | OpCode::BitAnd(_)
            | OpCode::BitOr(_)
            | OpCode::BitXor(_)
            | OpCode::BLeft(_)
            | OpCode::BRight(_)
            | OpCode::Ref(_)
            | OpCode::Store(_)
            | OpCode::AddS(_)
            | OpCode::SubS(_)
            | OpCode::MulS(_)
            | OpCode::DivS(_)
            | OpCode::RmdS(_)
            | OpCode::BAndS(_)
            | OpCode::BOrS(_)
            | OpCode::BXorS(_) => {
                let right = stack_pop(&mut stack);
                let left = stack_pop(&mut stack);
                let folded = eval_binary(op, left, right);
                stack.push(folded);
            }
        }
    }
}

fn const_prop_table(table: &mut OpCodeTable, arity_map: &HashMap<SmolStr, usize>) {
    let mut order: Vec<LocalAddr> = table.opcodes.keys().cloned().collect();
    order.sort_unstable_by_key(|addr| addr.offset);
    if order.is_empty() {
        return;
    }

    let mut offset_to_index = HashMap::new();
    for (idx, addr) in order.iter().enumerate() {
        offset_to_index.insert(addr.offset, idx);
    }

    let mut leaders: HashSet<usize> = HashSet::new();
    leaders.insert(0);
    for (idx, addr) in order.iter().enumerate() {
        let op = table.opcodes.get(addr).unwrap();
        if let Some(target) = jump_target(op)
            && let Some(target_idx) = offset_to_index.get(&target.offset)
        {
            leaders.insert(*target_idx);
        }
        if is_boundary(op) && idx + 1 < order.len() {
            leaders.insert(idx + 1);
        }
    }
    let mut leader_vec: Vec<usize> = leaders.into_iter().collect();
    leader_vec.sort_unstable();

    #[derive(Clone)]
    struct Block {
        start: usize,
        end: usize,
        succs: Vec<usize>,
    }

    let mut blocks: Vec<Block> = Vec::new();
    let mut instr_block = vec![0; order.len()];
    for (bi, start) in leader_vec.iter().enumerate() {
        let end = if bi + 1 < leader_vec.len() {
            leader_vec[bi + 1] - 1
        } else {
            order.len() - 1
        };
        for idx in *start..=end {
            instr_block[idx] = bi;
        }
        blocks.push(Block {
            start: *start,
            end,
            succs: Vec::new(),
        });
    }

    for bi in 0..blocks.len() {
        let end = blocks[bi].end;
        let addr = order[end];
        let op = table.opcodes.get(&addr).unwrap();
        let mut succs = Vec::new();
        match op {
            OpCode::Jump(_, _) | OpCode::LazyJump(_, _, _) => {
                if let Some(target) = jump_target(op)
                    && let Some(target_idx) = offset_to_index.get(&target.offset)
                {
                    succs.push(instr_block[*target_idx]);
                }
            }
            OpCode::JumpTrue(_, _, _) | OpCode::JumpFalse(_, _, _) => {
                if let Some(target) = jump_target(op)
                    && let Some(target_idx) = offset_to_index.get(&target.offset)
                {
                    succs.push(instr_block[*target_idx]);
                }
                if bi + 1 < blocks.len() {
                    succs.push(bi + 1);
                }
            }
            OpCode::Return(_) => {}
            _ => {
                if bi + 1 < blocks.len() {
                    succs.push(bi + 1);
                }
            }
        }
        succs.sort_unstable();
        succs.dedup();
        blocks[bi].succs = succs;
    }

    let mut in_env: Vec<Option<HashMap<DefaultKey, Option<Operand>>>> = vec![None; blocks.len()];
    in_env[0] = Some(HashMap::new());
    let mut worklist: VecDeque<usize> = VecDeque::new();
    worklist.push_back(0);

    while let Some(bi) = worklist.pop_front() {
        let env_in = in_env[bi].clone().unwrap_or_default();
        let env_out = analyze_block(
            table,
            &order,
            blocks[bi].start,
            blocks[bi].end,
            &env_in,
            arity_map,
        );

        for &succ in &blocks[bi].succs {
            let merged = match &in_env[succ] {
                Some(prev) => merge_env(prev, &env_out),
                None => env_out.clone(),
            };
            let changed = in_env[succ].as_ref().map_or(true, |prev| prev != &merged);
            if changed {
                in_env[succ] = Some(merged);
                worklist.push_back(succ);
            }
        }
    }

    for (bi, block) in blocks.iter().enumerate() {
        let env_in = in_env[bi].clone().unwrap_or_default();
        rewrite_block(table, &order, block.start, block.end, &env_in, arity_map);
    }
}

fn jump_target(op: &OpCode) -> Option<LocalAddr> {
    match op {
        OpCode::Jump(_, target)
        | OpCode::JumpTrue(_, target, _)
        | OpCode::JumpFalse(_, target, _)
        | OpCode::LazyJump(_, target, _) => *target,
        _ => None,
    }
}

pub(crate) fn const_prop_linear(code: &mut Code) {
    let mut arity_map: HashMap<SmolStr, usize> = HashMap::new();
    for func in &code.funcs {
        arity_map.insert(func.name.clone(), func.args);
    }

    for func in &mut code.funcs {
        if let Some(ref mut table) = func.codes {
            const_prop_table(table, &arity_map);
        }
    }
}

fn collect_local_reads(table: &OpCodeTable) -> HashSet<DefaultKey> {
    let mut reads = HashSet::new();
    for (_addr, op) in table.opcodes.iter() {
        match op {
            OpCode::StoreLocal(_, key, _) => {
                reads.insert(*key);
            }
            OpCode::Push(_, Operand::Val(key)) => {
                reads.insert(*key);
            }
            _ => {}
        }
    }
    reads
}

fn rebuild_locals(locals: &mut crate::compiler::ast::ssa_ir::LocalMap, dead: &HashSet<DefaultKey>) {
    let mut entries: Vec<(DefaultKey, usize)> =
        locals.locals.iter().map(|(k, v)| (*k, *v)).collect();
    entries.sort_unstable_by_key(|(_, index)| *index);

    let mut new_locals = BTreeMap::new();
    let mut next_index = 0;
    for (key, _index) in entries {
        if dead.contains(&key) {
            continue;
        }
        new_locals.insert(key, next_index);
        next_index += 1;
    }
    locals.locals = new_locals;
    locals.now_index = next_index;
}

fn is_pure_opcode(op: &OpCode) -> bool {
    match op {
        OpCode::Call(_, _)
        | OpCode::Jump(_, _)
        | OpCode::JumpTrue(_, _, _)
        | OpCode::JumpFalse(_, _, _)
        | OpCode::LazyJump(_, _, _)
        | OpCode::Return(_)
        | OpCode::LoadGlobal(_, _, _)
        | OpCode::SetArrayGlobal(_, _)
        | OpCode::SetArrayLocal(_, _) => false,
        _ => true,
    }
}

fn is_boundary(op: &OpCode) -> bool {
    matches!(
        op,
        OpCode::Call(_, _)
            | OpCode::Jump(_, _)
            | OpCode::JumpTrue(_, _, _)
            | OpCode::JumpFalse(_, _, _)
            | OpCode::LazyJump(_, _, _)
            | OpCode::Return(_)
    )
}

fn stack_effect(op: &OpCode) -> i32 {
    match op {
        OpCode::Push(_, _) | OpCode::StoreLocal(_, _, _) | OpCode::StoreGlobal(_, _, _) => 1,
        OpCode::LoadLocal(_, _, _) | OpCode::LoadGlobal(_, _, _) => -1,
        OpCode::LoadArrayLocal(_, _, len) | OpCode::LoadArrayGlobal(_, _, len) => -(*len as i32),
        OpCode::SetArrayLocal(_, _) | OpCode::SetArrayGlobal(_, _) => -2,
        OpCode::AIndex(_) | OpCode::Ref(_) => -1,
        OpCode::Not(_) | OpCode::Neg(_) | OpCode::Pos(_) | OpCode::SAdd(_) | OpCode::SSub(_) => 0,
        OpCode::Add(_)
        | OpCode::Sub(_)
        | OpCode::Mul(_)
        | OpCode::Div(_)
        | OpCode::Rmd(_)
        | OpCode::And(_)
        | OpCode::Or(_)
        | OpCode::Equ(_)
        | OpCode::NotEqu(_)
        | OpCode::BigEqu(_)
        | OpCode::LesEqu(_)
        | OpCode::Big(_)
        | OpCode::Less(_)
        | OpCode::BitAnd(_)
        | OpCode::BitOr(_)
        | OpCode::BitXor(_)
        | OpCode::BLeft(_)
        | OpCode::BRight(_)
        | OpCode::Store(_)
        | OpCode::AddS(_)
        | OpCode::SubS(_)
        | OpCode::MulS(_)
        | OpCode::DivS(_)
        | OpCode::RmdS(_)
        | OpCode::BAndS(_)
        | OpCode::BOrS(_)
        | OpCode::BXorS(_) => -1,
        OpCode::Pop(_, len) => -(*len as i32),
        OpCode::Call(_, _)
        | OpCode::Jump(_, _)
        | OpCode::JumpTrue(_, _, _)
        | OpCode::JumpFalse(_, _, _)
        | OpCode::LazyJump(_, _, _)
        | OpCode::Return(_)
        | OpCode::Nop(_) => 0,
    }
}

fn statement_is_dead_definition(stmt: &[(LocalAddr, OpCode)], dead: &HashSet<DefaultKey>) -> bool {
    let Some((_addr, last)) = stmt.last() else {
        return false;
    };
    let is_dead_store = match last {
        OpCode::LoadLocal(_, key, _) | OpCode::LoadArrayLocal(_, key, _) => dead.contains(key),
        _ => false,
    };
    if !is_dead_store {
        return false;
    }
    stmt.iter().all(|(_addr, op)| is_pure_opcode(op))
}

fn eliminate_dead_locals_in_table(table: &mut OpCodeTable, dead: &HashSet<DefaultKey>) {
    let mut kept = OpCodeTable::new();
    let mut current_stmt: Vec<(LocalAddr, OpCode)> = Vec::new();
    let mut depth: i32 = 0;

    for (addr, op) in table.opcodes.iter() {
        current_stmt.push((*addr, op.clone()));
        depth += stack_effect(op);
        if depth < 0 {
            depth = 0;
        }

        if is_boundary(op) {
            for (a, o) in current_stmt.drain(..) {
                kept.opcodes.insert(a, o);
            }
            depth = 0;
            continue;
        }

        if depth == 0 {
            if !statement_is_dead_definition(&current_stmt, dead) {
                for (a, o) in current_stmt.drain(..) {
                    kept.opcodes.insert(a, o);
                }
            } else {
                current_stmt.clear();
            }
        }
    }

    for (a, o) in current_stmt.drain(..) {
        kept.opcodes.insert(a, o);
    }

    let mut new_table = OpCodeTable::new();
    new_table.append_code(&kept);
    *table = new_table;
}

pub(crate) fn eliminate_dead_locals(code: &mut Code) {
    for func in &mut code.funcs {
        let Some(ref mut table) = func.codes else {
            continue;
        };
        let reads = collect_local_reads(table);
        let mut dead = HashSet::new();
        for key in func.locals.locals.keys() {
            if !reads.contains(key) {
                dead.insert(*key);
            }
        }
        if dead.is_empty() {
            continue;
        }
        eliminate_dead_locals_in_table(table, &dead);
        rebuild_locals(&mut func.locals, &dead);
    }
}
