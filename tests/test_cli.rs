use expect_test::{expect, Expect};
use std::io::Write;
use std::process::{Command, Stdio};

fn run_source(buf: &[u8]) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_openex"))
        .arg("--cli")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn process");
    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(buf)
            .unwrap();
    }
    let output = child.wait_with_output().unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success());
    assert!(stderr.is_empty());
    stdout
}

#[track_caller]
fn check(buf: &[u8], expect: Expect) {
    expect.assert_eq(run_source(buf).as_str());
}

#[test]
pub fn test_var_define() {
    check(b"import system;\
    var a = 3.1415926535;\
    var b = 4;\
    b = b * 3;\
    system.println(a);\
    system.println(b);",expect![[r#"
        > 3.1415926535
        12
    "#]]);
}

#[test]
pub fn test_loop() {
    check(b"import system;\
    var a = 0;\
    while(a < 10) {\
    system.println(\"number: \" + a);\
    a++;\
    if (a > 5) {\
    continue;\
    }\
    system.println(\"a > 5\");\
    }",expect![[r#"
        > number: 0
        a > 5
        number: 1
        a > 5
        number: 2
        a > 5
        number: 3
        a > 5
        number: 4
        a > 5
        number: 5
        number: 6
        number: 7
        number: 8
        number: 9
    "#]]);
}

#[test]
pub fn test_judgment() {
    check(b"import system;\
    var a = 3 + 1 - 4;\
    if (a != 0) {\
    system.println(\"Hello!\"); \
    }elif (a == 0) {\
    system.println(\"a is zero.\");\
    }",expect![[r#"
        > a is zero.
    "#]]);
}

#[test]
pub fn test_fib() {
    check(b"import system;\
    function fib(n) {\
    if (n < 2) {\
    return n;\
    } else {\
    return this.fib(n - 1) + this.fib(n - 2);\
    }}\
    system.println(this.fib(30));",expect![[r#"
        > 832040
    "#]]);
    // fib(30) == 832040
    // fib(35) == 9227465
    // fib(40) == 102334155
}

#[test]
pub fn test_fib_2() {
    check(b"import system;\
     function fib(n) {\
    if (n < 2) {\
    return n;\
    }\
    var a = 0;\
    var b = 1;\
    var i = 2;\
    while (i <= n) {\
        var next = a + b;\
        a = b;\
        b = next;\
        i++;\
    }\
    return b;\
    }\
    system.println(this.fib(40));", expect![[r#"
        > 102334155
    "#]])
}
