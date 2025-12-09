use expect_test::{Expect, expect};

use crate::compiler::{file::SourceFile, parser::Parser};

fn parse_source(src: &str) -> Result<crate::compiler::ast::ASTStmtTree, super::ParserError> {
    let mut file = SourceFile::new("<test_input>".into(), src.into(), Default::default());
    let parser = Parser::new(&mut file);
    let ast = parser.parser();
    ast
}

#[track_caller]
fn check(src: &str, expect: Expect) {
    let ast = parse_source(src).unwrap();
    expect.assert_debug_eq(&ast);
}

#[test]
fn basic_work() {
    check(r#"import system; system.println() + 3;"#, expect![[r#"
        Root(
            [
                Import(
                    Token {
                        line: 0,
                        column: 8,
                        t_type: Identifier,
                        index: 8,
                        data: "system",
                    },
                ),
                Expr(
                    Expr {
                        token: Token {
                            line: 0,
                            column: 33,
                            t_type: Operator,
                            index: 33,
                            data: "+",
                        },
                        op: Add,
                        left: Expr {
                            token: Token {
                                line: 0,
                                column: 22,
                                t_type: Operator,
                                index: 22,
                                data: ".",
                            },
                            op: Ref,
                            left: Var(
                                Token {
                                    line: 0,
                                    column: 16,
                                    t_type: Identifier,
                                    index: 16,
                                    data: "system",
                                },
                            ),
                            right: Call {
                                name: Var(
                                    Token {
                                        line: 0,
                                        column: 23,
                                        t_type: Identifier,
                                        index: 23,
                                        data: "println",
                                    },
                                ),
                                args: [],
                            },
                        },
                        right: Literal(
                            Token {
                                line: 0,
                                column: 35,
                                t_type: Number,
                                index: 35,
                                data: "3",
                            },
                        ),
                    },
                ),
            ],
        )
    "#]]);
}

#[test]
fn vars() {
    check(r#"
var num = .5138;
var num1 = 1.2e-3;
var num2 = 10.;
var num3 = 3.14159265358979;
num + 34;"#,
    expect![[r#"
        Root(
            [
                Var {
                    name: Token {
                        line: 1,
                        column: 5,
                        t_type: Identifier,
                        index: 6,
                        data: "num",
                    },
                    value: Some(
                        Literal(
                            Token {
                                line: 1,
                                column: 11,
                                t_type: Float,
                                index: 12,
                                data: ".5138",
                            },
                        ),
                    ),
                },
                Var {
                    name: Token {
                        line: 2,
                        column: 5,
                        t_type: Identifier,
                        index: 23,
                        data: "num1",
                    },
                    value: Some(
                        Literal(
                            Token {
                                line: 2,
                                column: 12,
                                t_type: Float,
                                index: 30,
                                data: "1.2e-3",
                            },
                        ),
                    ),
                },
                Var {
                    name: Token {
                        line: 3,
                        column: 5,
                        t_type: Identifier,
                        index: 42,
                        data: "num2",
                    },
                    value: Some(
                        Literal(
                            Token {
                                line: 3,
                                column: 12,
                                t_type: Float,
                                index: 49,
                                data: "10.",
                            },
                        ),
                    ),
                },
                Var {
                    name: Token {
                        line: 4,
                        column: 5,
                        t_type: Identifier,
                        index: 58,
                        data: "num3",
                    },
                    value: Some(
                        Literal(
                            Token {
                                line: 4,
                                column: 12,
                                t_type: Float,
                                index: 65,
                                data: "3.14159265358979",
                            },
                        ),
                    ),
                },
                Expr(
                    Expr {
                        token: Token {
                            line: 5,
                            column: 5,
                            t_type: Operator,
                            index: 87,
                            data: "+",
                        },
                        op: Add,
                        left: Var(
                            Token {
                                line: 5,
                                column: 1,
                                t_type: Identifier,
                                index: 83,
                                data: "num",
                            },
                        ),
                        right: Literal(
                            Token {
                                line: 5,
                                column: 7,
                                t_type: Number,
                                index: 89,
                                data: "34",
                            },
                        ),
                    },
                ),
            ],
        )
    "#]]);
}
