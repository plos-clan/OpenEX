use crate::compiler::ast::ExprOp;
use crate::compiler::ast::ssa_ir::Operand::{ImmBool, ImmFlot, ImmNum, ImmStr, Library, Reference};
use crate::compiler::ast::ssa_ir::{Code, LocalAddr, OpCode, OpCodeTable, Operand};
use dashu::float::{Context, DBig};
use slotmap::DefaultKey;
use smol_str::{SmolStr, SmolStrBuilder};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

pub fn unary_optimizer(op: ExprOp, operand: &Operand) -> Option<Operand> {
    match op {
        ExprOp::Not => fold_not(operand),
        ExprOp::SAdd => fold_num_unary(operand, |n| n + 1, |f| f.clone() + DBig::from(1)),
        ExprOp::SSub => fold_num_unary(operand, |n| n - 1, |f| f.clone() - DBig::from(1)),
        ExprOp::Neg => fold_num_unary(operand, |n| -n, |f| -f.clone()),
        _ => None,
    }
}

pub fn expr_optimizer(left: &Operand, right: &Operand, op: ExprOp) -> Option<Operand> {
    match op {
        ExprOp::Add => fold_num_bin(
            left,
            right,
            |a, b| a + b,
            |a, b| a.clone() + b.clone(),
            |a, b| a + b,
        ),
        ExprOp::Sub => fold_num_bin(
            left,
            right,
            |a, b| a - b,
            |a, b| a.clone() - b.clone(),
            |a, b| a - b,
        ),
        ExprOp::Mul => fold_num_bin(
            left,
            right,
            |a, b| a * b,
            |a, b| a.clone() * b.clone(),
            |a, b| a * b,
        ),
        ExprOp::Div => fold_num_bin(
            left,
            right,
            |a, b| a / b,
            |a, b| a.clone() / b.clone(),
            |a, b| {
                let context = Context::new(30);
                context.div(a.repr(), b.repr()).value()
            },
        ),
        ExprOp::Rmd => fold_num_bin(
            left,
            right,
            |a, b| a % b,
            |a, b| a.clone() % b.clone(),
            |a, b| {
                let context = Context::new(30);
                context.rem(a.repr(), b.repr()).value()
            },
        ),
        ExprOp::BitAnd => fold_bit_op(left, right, |a, b| a & b),
        ExprOp::BitOr => fold_bit_op(left, right, |a, b| a | b),
        ExprOp::BitXor => fold_bit_op(left, right, |a, b| a ^ b),
        ExprOp::BLeft => fold_bit_op(left, right, |a, b| a << b),
        ExprOp::BRight => fold_bit_op(left, right, |a, b| a >> b),
        ExprOp::Big => fold_num_cmp(left, right, |a, b| a > b, |a, b| a > b),
        ExprOp::Less => fold_num_cmp(left, right, |a, b| a < b, |a, b| a < b),
        ExprOp::BigEqu => fold_num_cmp(left, right, |a, b| a >= b, |a, b| a >= b),
        ExprOp::LesEqu => fold_num_cmp(left, right, |a, b| a <= b, |a, b| a <= b),
        ExprOp::Equ => fold_eq(left, right, false),
        ExprOp::NotEqu => fold_eq(left, right, true),
        ExprOp::And => fold_bool_op(left, right, |a, b| a && b),
        ExprOp::Or => fold_bool_op(left, right, |a, b| a || b),
        ExprOp::Ref => fold_ref(left, right),
        _ => None,
    }
}

fn fold_not(operand: &Operand) -> Option<Operand> {
    if let ImmBool(b) = operand {
        Some(ImmBool(!b))
    } else {
        None
    }
}

fn fold_num_unary<F>(operand: &Operand, num_op: fn(i64) -> i64, float_op: F) -> Option<Operand>
where
    F: Fn(&DBig) -> DBig,
{
    if let ImmNum(num) = operand {
        Some(ImmNum(num_op(*num)))
    } else if let ImmFlot(flot) = operand {
        Some(ImmFlot(float_op(flot)))
    } else {
        None
    }
}

fn fold_num_bin<F, G>(
    left: &Operand,
    right: &Operand,
    int_op: fn(i64, i64) -> i64,
    float_op: F,
    mixed_op: G,
) -> Option<Operand>
where
    F: Fn(&DBig, &DBig) -> DBig,
    G: Fn(DBig, DBig) -> DBig,
{
    match (left, right) {
        (ImmNum(a), ImmNum(b)) => Some(ImmNum(int_op(*a, *b))),
        (ImmFlot(a), ImmFlot(b)) => Some(ImmFlot(float_op(a, b))),
        (ImmNum(a), ImmFlot(b)) => Some(ImmFlot(mixed_op(DBig::from(*a), b.clone()))),
        (ImmFlot(a), ImmNum(b)) => Some(ImmFlot(mixed_op(a.clone(), DBig::from(*b)))),
        _ => None,
    }
}

fn fold_num_cmp<F, G>(
    left: &Operand,
    right: &Operand,
    int_cmp: F,
    float_cmp: G,
) -> Option<Operand>
where
    F: Fn(i64, i64) -> bool,
    G: Fn(&DBig, &DBig) -> bool,
{
    match (left, right) {
        (ImmNum(a), ImmNum(b)) => Some(ImmBool(int_cmp(*a, *b))),
        (ImmFlot(a), ImmFlot(b)) => Some(ImmBool(float_cmp(a, b))),
        _ => None,
    }
}

fn fold_bit_op(left: &Operand, right: &Operand, op: fn(i64, i64) -> i64) -> Option<Operand> {
    if let (ImmNum(a), ImmNum(b)) = (left, right) {
        Some(ImmNum(op(*a, *b)))
    } else {
        None
    }
}

fn fold_bool_op(left: &Operand, right: &Operand, op: fn(bool, bool) -> bool) -> Option<Operand> {
    if let (ImmBool(a), ImmBool(b)) = (left, right) {
        Some(ImmBool(op(*a, *b)))
    } else {
        None
    }
}

fn fold_eq(left: &Operand, right: &Operand, negate: bool) -> Option<Operand> {
    let eq = match (left, right) {
        (ImmNum(a), ImmNum(b)) => Some(a == b),
        (ImmFlot(a), ImmFlot(b)) => Some(a == b),
        (ImmBool(a), ImmBool(b)) => Some(a == b),
        (ImmStr(a), ImmStr(b)) => Some(a == b),
        _ => None,
    }?;
    Some(ImmBool(if negate { !eq } else { eq }))
}

fn fold_ref(left: &Operand, right: &Operand) -> Option<Operand> {
    if let (Reference(str) | Library(str), Reference(str1) | Library(str1)) = (left, right) {
        let mut ref_build = SmolStrBuilder::new();
        ref_build.push_str(str1.as_str());
        ref_build.push('/');
        ref_build.push_str(str.as_str());
        Some(Reference(ref_build.finish()))
    } else {
        None
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

fn fold_unary(op: &OpCode, operand: &Operand) -> Option<Operand> {
    match op {
        OpCode::Not(_) => fold_not(operand),
        OpCode::SAdd(_) => fold_num_unary(operand, |n| n + 1, |f| f.clone() + DBig::from(1)),
        OpCode::SSub(_) => fold_num_unary(operand, |n| n - 1, |f| f.clone() - DBig::from(1)),
        OpCode::Neg(_) => fold_num_unary(operand, |n| -n, |f| -f.clone()),
        _ => None,
    }
}

fn fold_binary(op: &OpCode, left: &Operand, right: &Operand) -> Option<Operand> {
    match op {
        OpCode::Add(_) => fold_num_bin(
            left,
            right,
            |a, b| a + b,
            |a, b| a.clone() + b.clone(),
            |a, b| a + b,
        ),
        OpCode::Sub(_) => fold_num_bin(
            left,
            right,
            |a, b| a - b,
            |a, b| a.clone() - b.clone(),
            |a, b| a - b,
        ),
        OpCode::Mul(_) => fold_num_bin(
            left,
            right,
            |a, b| a * b,
            |a, b| a.clone() * b.clone(),
            |a, b| a * b,
        ),
        OpCode::Div(_) => fold_num_bin(
            left,
            right,
            |a, b| a / b,
            |a, b| a.clone() / b.clone(),
            |a, b| {
                let context = Context::new(30);
                context.div(a.repr(), b.repr()).value()
            },
        ),
        OpCode::Rmd(_) => fold_num_bin(
            left,
            right,
            |a, b| a % b,
            |a, b| a.clone() % b.clone(),
            |a, b| {
                let context = Context::new(30);
                context.rem(a.repr(), b.repr()).value()
            },
        ),
        OpCode::BitAnd(_) => fold_bit_op(left, right, |a, b| a & b),
        OpCode::BitOr(_) => fold_bit_op(left, right, |a, b| a | b),
        OpCode::BitXor(_) => fold_bit_op(left, right, |a, b| a ^ b),
        OpCode::BLeft(_) => fold_bit_op(left, right, |a, b| a << b),
        OpCode::BRight(_) => fold_bit_op(left, right, |a, b| a >> b),
        OpCode::Big(_) => fold_num_cmp(left, right, |a, b| a > b, |a, b| a > b),
        OpCode::Less(_) => fold_num_cmp(left, right, |a, b| a < b, |a, b| a < b),
        OpCode::BigEqu(_) => fold_num_cmp(left, right, |a, b| a >= b, |a, b| a >= b),
        OpCode::LesEqu(_) => fold_num_cmp(left, right, |a, b| a <= b, |a, b| a <= b),
        OpCode::Equ(_) => fold_eq(left, right, false),
        OpCode::NotEqu(_) => fold_eq(left, right, true),
        OpCode::And(_) => fold_bool_op(left, right, |a, b| a && b),
        OpCode::Or(_) => fold_bool_op(left, right, |a, b| a || b),
        OpCode::Ref(_) => fold_ref(left, right),
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

fn const_prop_step(
    op: &OpCode,
    env: &mut HashMap<DefaultKey, Option<Operand>>,
    stack: &mut Vec<Option<Operand>>,
    arity_map: &HashMap<SmolStr, usize>,
) -> Option<Operand> {
    match op {
        OpCode::Push(_, imm) => {
            stack_push_operand(stack, imm);
            None
        }
        OpCode::Pop(_, len) => {
            stack_pop_n(stack, *len);
            None
        }
        OpCode::StoreLocal(_, key, _) => {
            if let Some(Some(constant)) = env.get(key) {
                stack.push(Some(constant.clone()));
                Some(constant.clone())
            } else {
                stack.push(None);
                None
            }
        }
        OpCode::AddLocalImm(_, key, imm) => {
            if let Some(Some(constant)) = env.get(key) {
                let next = match constant {
                    ImmNum(v) => Some(ImmNum(*v + *imm)),
                    ImmFlot(v) => Some(ImmFlot(v + DBig::from(*imm))),
                    _ => None,
                };
                env.insert(*key, next.clone());
            } else {
                env.insert(*key, None);
            }
            None
        }
        OpCode::LoadLocal(_, key, _) => {
            let value = stack_pop(stack);
            env.insert(*key, value);
            None
        }
        OpCode::StoreGlobal(_, _, _) => {
            stack.push(None);
            None
        }
        OpCode::LoadGlobal(_, _, _) => {
            let _ = stack_pop(stack);
            None
        }
        OpCode::LoadArrayLocal(_, key, len) => {
            stack_pop_n(stack, *len);
            env.insert(*key, None);
            None
        }
        OpCode::LoadArrayGlobal(_, _, len) => {
            stack_pop_n(stack, *len);
            None
        }
        OpCode::SetArrayLocal(_, key) => {
            stack_pop_n(stack, 2);
            env.insert(*key, None);
            None
        }
        OpCode::SetArrayGlobal(_, _) => {
            stack_pop_n(stack, 2);
            None
        }
        OpCode::AIndex(_) => {
            stack_pop_n(stack, 2);
            stack.push(None);
            None
        }
        OpCode::Call(_, name) => {
            if let Some(arity) = arity_map.get(name) {
                stack_pop_n(stack, *arity + 1);
            } else {
                stack.clear();
            }
            stack.push(None);
            None
        }
        OpCode::JumpTrue(_, _, _) | OpCode::JumpFalse(_, _, _) => {
            let _ = stack_pop(stack);
            None
        }
        OpCode::Jump(_, _) | OpCode::LazyJump(_, _, _) | OpCode::Return(_) => None,
        OpCode::Nop(_) => None,
        OpCode::Not(_)
        | OpCode::Neg(_)
        | OpCode::Pos(_)
        | OpCode::SAdd(_)
        | OpCode::SSub(_) => {
            let value = stack_pop(stack);
            let folded = eval_unary(op, value);
            stack.push(folded);
            None
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
            let right = stack_pop(stack);
            let left = stack_pop(stack);
            let folded = eval_binary(op, left, right);
            stack.push(folded);
            None
        }
    }
}

fn eval_unary(op: &OpCode, value: Option<Operand>) -> Option<Operand> {
    let operand = value?;
    fold_unary(op, &operand)
}

fn eval_binary(op: &OpCode, left: Option<Operand>, right: Option<Operand>) -> Option<Operand> {
    let left = left?;
    let right = right?;
    fold_binary(op, &left, &right)
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
        let _ = const_prop_step(op, &mut env, &mut stack, arity_map);
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
        let replacement = {
            let op_ref: &OpCode = &*op;
            const_prop_step(op_ref, &mut env, &mut stack, arity_map)
        };
        if let Some(constant) = replacement {
            replace_with_push(op, constant);
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

fn local_arith_peephole_table(table: &mut OpCodeTable) {
    if table.opcodes.is_empty() {
        return;
    }
    let mut order: Vec<LocalAddr> = table.opcodes.keys().cloned().collect();
    order.sort_unstable_by_key(|addr| addr.offset);

    let mut jump_targets: HashSet<LocalAddr> = HashSet::new();
    for (_addr, op) in table.opcodes.iter() {
        if let Some(target) = jump_target(op) {
            jump_targets.insert(target);
        }
    }

    let mut kept = OpCodeTable::new();
    let mut i = 0;
    while i < order.len() {
        let addr0 = order[i];
        let op0 = table.opcodes.get(&addr0).unwrap();

        if let OpCode::StoreLocal(_, key0, _) = op0 {
            if let (Some(addr1), Some(addr2)) = (order.get(i + 1), order.get(i + 2)) {
                if !jump_targets.contains(addr1) && !jump_targets.contains(addr2) {
                    let op1 = table.opcodes.get(addr1).unwrap();
                    let op2 = table.opcodes.get(addr2).unwrap();
                    if let OpCode::LoadLocal(_, key2, _) = op2
                        && key2 == key0
                    {
                        let imm = match op1 {
                            OpCode::SAdd(_) => Some(1),
                            OpCode::SSub(_) => Some(-1),
                            _ => None,
                        };
                        if let Some(imm) = imm {
                            kept.opcodes.insert(
                                addr0,
                                OpCode::AddLocalImm(Some(addr0), *key0, imm),
                            );
                            i += 3;
                            continue;
                        }
                    }
                }
            }

            if let (Some(addr1), Some(addr2), Some(addr3)) =
                (order.get(i + 1), order.get(i + 2), order.get(i + 3))
            {
                if !jump_targets.contains(addr1)
                    && !jump_targets.contains(addr2)
                    && !jump_targets.contains(addr3)
                {
                    let op1 = table.opcodes.get(addr1).unwrap();
                    let op2 = table.opcodes.get(addr2).unwrap();
                    let op3 = table.opcodes.get(addr3).unwrap();
                    if let (OpCode::Push(_, Operand::ImmNum(imm)), OpCode::LoadLocal(_, key3, _)) =
                        (op1, op3)
                    {
                        if key3 == key0 {
                            let delta = match op2 {
                                OpCode::Add(_) => Some(*imm),
                                OpCode::Sub(_) => Some(-*imm),
                                _ => None,
                            };
                            if let Some(delta) = delta {
                                kept.opcodes.insert(
                                    addr0,
                                    OpCode::AddLocalImm(Some(addr0), *key0, delta),
                                );
                                i += 4;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        if let OpCode::Push(_, Operand::ImmNum(imm)) = op0 {
            if let (Some(addr1), Some(addr2), Some(addr3)) =
                (order.get(i + 1), order.get(i + 2), order.get(i + 3))
            {
                if !jump_targets.contains(addr1)
                    && !jump_targets.contains(addr2)
                    && !jump_targets.contains(addr3)
                {
                    let op1 = table.opcodes.get(addr1).unwrap();
                    let op2 = table.opcodes.get(addr2).unwrap();
                    let op3 = table.opcodes.get(addr3).unwrap();
                    if let (OpCode::StoreLocal(_, key1, _), OpCode::Add(_), OpCode::LoadLocal(_, key3, _)) =
                        (op1, op2, op3)
                    {
                        if key1 == key3 {
                            kept.opcodes.insert(
                                addr0,
                                OpCode::AddLocalImm(Some(addr0), *key1, *imm),
                            );
                            i += 4;
                            continue;
                        }
                    }
                }
            }
        }

        kept.opcodes.insert(addr0, op0.clone());
        i += 1;
    }

    let mut new_table = OpCodeTable::new();
    new_table.append_code(&kept);
    *table = new_table;
}

pub(crate) fn local_arith_peephole(code: &mut Code) {
    for func in &mut code.funcs {
        if let Some(ref mut table) = func.codes {
            local_arith_peephole_table(table);
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
        OpCode::Not(_)
        | OpCode::Neg(_)
        | OpCode::Pos(_)
        | OpCode::SAdd(_)
        | OpCode::SSub(_)
        | OpCode::AddLocalImm(_, _, _) => 0,
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
