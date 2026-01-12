use crate::compiler::ast::ssa_ir::{OpCode, OpCodeTable};
use crate::compiler::lexer::Token;
use crate::compiler::parser::ParserError;
use crate::compiler::parser::symbol_table::ContextType;
use crate::compiler::semantic::Semantic;

pub fn loop_back_semantic(
    semantic: &mut Semantic,
    is_break: bool,
    token: Token,
) -> Result<OpCodeTable, ParserError> {
    semantic
        .compiler_data()
        .symbol_table
        .get_context(&ContextType::Loop)
        .map_or_else(
            || Err(ParserError::BackOutsideLoop(token)),
            |_context| {
                let mut table = OpCodeTable::new();
                table.add_opcode(OpCode::LazyJump(None, None, is_break));
                Ok(table)
            },
        )
}
