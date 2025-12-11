use crate::compiler::ast::ssa_ir::{Code, Function, OpCode, OpCodeTable, Operand, ValueGuessType};
use crate::compiler::ast::{ASTExprTree, ASTStmtTree};
use crate::compiler::lexer::Token;
use crate::compiler::parser::symbol_table::ElementType::Argument;
use crate::compiler::parser::symbol_table::{ContextType, ElementType};
use crate::compiler::parser::ParserError;
use crate::compiler::parser::ParserError::NoNativeImplement;
use crate::compiler::semantic::block::block_semantic;
use crate::compiler::semantic::Semantic;
use crate::library::find_library;
use smol_str::SmolStr;

pub fn native_function_semantic(
    semantic: &mut Semantic,
    mut name: Token,
    arguments: Vec<ASTExprTree>,
    code: &mut Code,
) -> Result<(), ParserError> {
    let mut lib_name = semantic.file.name.clone();
    lib_name = lib_name.split('.').next().unwrap().to_string();
    let func_name = name.value::<SmolStr>().clone().unwrap();

    if code.find_function(func_name.clone()).is_some() {
        return Err(ParserError::SymbolDefined(name));
    }

    semantic
        .compiler_data()
        .symbol_table
        .add_element(func_name.clone(), ElementType::Function(arguments.len()));

    find_library(lib_name.as_str(), |module| {
        if let Some(lib) = module
            && let Some(func) = lib.find_func(&func_name)
            && arguments.len() == func.arity
        {
            code.add_function(Function {
                name: func_name,
                args: func.arity,
                codes: None,
            });
            Ok(())
        } else {
            Err(NoNativeImplement(name))
        }
    })
}

pub fn function_semantic(
    semantic: &mut Semantic,
    mut name: Token,
    arguments: Vec<ASTExprTree>,
    body: Vec<ASTStmtTree>,
    code: &mut Code,
) -> Result<(), ParserError> {
    let func_name = name.value::<SmolStr>().clone().unwrap();
    if code.find_function(func_name.clone()).is_some() {
        return Err(ParserError::SymbolDefined(name));
    }
    semantic
        .compiler_data()
        .symbol_table
        .add_context(ContextType::Func);
    let mut tables = OpCodeTable::new();

    let args_len = arguments.len();
    for i in arguments {
        if let ASTExprTree::Var(token) = i {
            let mut token_c = token.clone();
            let key = code.alloc_value(token, ValueGuessType::Unknown);
            semantic
                .compiler_data()
                .symbol_table
                .add_element(token_c.value().unwrap(), Argument);
            tables.add_opcode(OpCode::LoadLocal(None, key, Operand::Val(key)));
        }
    }

    let mut blk = block_semantic(semantic, body, code)?;
    tables.append_code(&mut blk);

    semantic.compiler_data().symbol_table.exit_context();

    code.add_function(Function {
        name: func_name,
        args: args_len,
        codes: Some(tables),
    });
    Ok(())
}
