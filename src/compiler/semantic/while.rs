use crate::compiler::ast::ssa_ir::{Code, LocalMap, OpCode, OpCodeTable, Operand, ValueGuessType};
use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::lints::Lint::LoopNoExpr;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::block::block_semantic;
use crate::compiler::semantic::expression::lower_expr;
use crate::compiler::semantic::Semantic;
use crate::compiler::Compiler;

pub fn while_semantic(
    semantic: &mut Semantic,
    expr: &ASTExprTree,
    body: Vec<ASTStmtTree>,
    code: &mut Code,
    locals:&mut LocalMap
) -> Result<OpCodeTable, ParserError> {
    let exp = lower_expr(semantic, expr, code, None)?;
    let mut code_table = OpCodeTable::new();

    if exp.1 != ValueGuessType::Bool {
        return Err(ParserError::IllegalTypeCombination(expr.clone().token().clone()));
    }

    if matches!(exp.0,Operand::ImmBool(_)) && !semantic.file.has_warnings(LoopNoExpr) {
        Compiler::warning_info_expr(
            semantic.file,
            "'while(true)' can be written as 'while'.",
            expr,
            LoopNoExpr,
        );
    }
    let start = code_table.append_code(&exp.2).0;
    let k = code_table.add_opcode(OpCode::JumpTrue(None,None, exp.0));

    let blk_table = block_semantic(semantic, body, code, locals)?;
    code_table.append_code(&blk_table);
    code_table.add_opcode(OpCode::Jump(None, Some(start)));
    let end_addr = code_table.add_opcode(OpCode::Nop(None));

    if let Some(jump_true_op) = code_table.find_code_mut(k)
        && let OpCode::JumpTrue(_,target, _) = jump_true_op
    {
        *target = Some(end_addr);
    }

    Ok(code_table)
}
