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
    "#]])
}

#[test]
pub fn test_loop() {
    check(b"import system;\
    var a = 0;\
    while(a < 5) {\
    system.println(\"number: \" + a);\
    a++;\
    }",expect![[r#"
        > number: 0
        number: 1
        number: 2
        number: 3
        number: 4
        number: 5
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
