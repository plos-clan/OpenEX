use crate::compiler::ast::ssa_ir::{LocalMap, OpCode, OpCodeTable, Operand, ValueAlloc, ValueGuessType};
use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::lints::Lint::LoopNoExpr;
use crate::compiler::parser::symbol_table::ContextType;
use crate::compiler::parser::ParserError;
use crate::compiler::semantic::block::block_semantic;
use crate::compiler::semantic::expression::lower_expr;
use crate::compiler::semantic::Semantic;
use crate::compiler::Compiler;

pub fn while_semantic(
    semantic: &mut Semantic,
    expr: &ASTExprTree,
    body: Vec<ASTStmtTree>,
    code: &mut ValueAlloc,
    locals: &mut LocalMap,
    is_easy: bool,
) -> Result<OpCodeTable, ParserError> {
    semantic
        .compiler_data()
        .symbol_table
        .add_context(ContextType::Loop);

    let exp = lower_expr(semantic, expr, code, None)?;
    let mut code_table = OpCodeTable::new();

    if exp.1 != ValueGuessType::Bool {
        return Err(ParserError::IllegalTypeCombination(
            expr.clone().token().clone(),
        ));
    }

    if matches!(exp.0, Operand::ImmBool(_)) && !semantic.file.has_warnings(LoopNoExpr) && !is_easy {
        Compiler::warning_info_expr(
            semantic.file,
            "'while(true)' can be written as 'while'.",
            expr,
            LoopNoExpr,
        );
    }

    let start = code_table.add_opcode(OpCode::Nop(None));
    code_table.append_code(&exp.2);
    let k = code_table.add_opcode(OpCode::JumpFalse(None, None, exp.0));

    let blk_table = block_semantic(semantic, body, code, locals)?;
    code_table.append_code(&blk_table);
    code_table.add_opcode(OpCode::Jump(None, Some(start)));
    let end_addr = code_table.add_opcode(OpCode::Nop(None));

    code_table.change_code(|codes| {
        for code in &mut codes.opcodes {
            if let OpCode::LazyJump(_addr, target_addr, is_break) = code.1 {
                *target_addr = if *is_break {
                    Some(end_addr)
                } else {
                    Some(start)
                };
            }
        }
    });

    if let Some(jump_true_op) = code_table.find_code_mut(k)
        && let OpCode::JumpFalse(_, target, _) = jump_true_op
    {
        *target = Some(end_addr);
    }
    semantic.compiler_data().symbol_table.exit_context();

    Ok(code_table)
}
