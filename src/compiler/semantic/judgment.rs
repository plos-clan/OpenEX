use crate::compiler::ast::ssa_ir::{Code, LocalMap, OpCode, OpCodeTable, ValueGuessType};
use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::block::block_semantic;
use crate::compiler::semantic::expression::lower_expr;
use crate::compiler::semantic::Semantic;

pub fn judgment_semantic(
    semantic: &mut Semantic,
    expr: &ASTExprTree,
    then_body: Vec<ASTStmtTree>,
    else_body: Vec<ASTStmtTree>,
    code: &mut Code,
    locals: &mut LocalMap,
) -> Result<OpCodeTable, ParserError> {
    let exp = lower_expr(semantic, expr, code, None)?;
    let mut code_table = OpCodeTable::new();

    if exp.1 != ValueGuessType::Bool {
        return Err(ParserError::IllegalTypeCombination(
            expr.clone().token().clone(),
        ));
    }

    code_table.append_code(&exp.2);
    let k = code_table.add_opcode(OpCode::JumpFalse(None, None, exp.0));

    let blk_table = block_semantic(semantic, then_body, code, locals)?;
    code_table.append_code(&blk_table);
    let jump_else = code_table.add_opcode(OpCode::Jump(None, None));

    if else_body.is_empty() {
        let end_addr = code_table.add_opcode(OpCode::Nop(None));
        if let Some(jump_true_op) = code_table.find_code_mut(k)
            && let OpCode::JumpFalse(_, target, _) = jump_true_op
        {
            *target = Some(end_addr);
        }
        if let Some(jump_op) = code_table.find_code_mut(jump_else)
            && let OpCode::Jump(_, target) = jump_op
        {
            *target = Some(end_addr);
        }
    } else {
        let else_table = block_semantic(semantic, else_body, code, locals)?;
        let end_addr = code_table.append_code(&else_table).0;
        let end_else = code_table.add_opcode(OpCode::Nop(None));

        if let Some(jump_true_op) = code_table.find_code_mut(k)
            && let OpCode::JumpFalse(_, target, _) = jump_true_op
        {
            *target = Some(end_addr);
        }

        if let Some(jump_op) = code_table.find_code_mut(jump_else)
            && let OpCode::Jump(_, target) = jump_op
        {
            *target = Some(end_else);
        }
    }

    Ok(code_table)
}
