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

/// 考拉兹猜想测试
#[test]
pub fn test_collatz() {
    check(b"import system;\
    function test_collatz(n) {\
    var steps = 0;\
    while (n != 1) {\
    if (n % 2 == 0) {\
    n = n / 2;\
    } else {\
    n = 3 * n + 1;\
    }\
    steps = steps + 1;\
    }\
    return steps;\
    }\
    var result = this.test_collatz(7);\
    system.println(result);",expect![[r#"
        > 16
    "#]]);
}

/// 质数计数器
#[test]
pub fn test_count_primes() {
    check(b"import system;\
    function count_primes(limit) {\
    var count = 0;\
    var num = 2;\
    while (num <= limit) {\
    var is_prime = 1;\
    var i = 2;\
    while (i * i <= num) {\
    if (num % i == 0) {\
    is_prime = 0;\
    }i = i + 1;}\
    if (is_prime == 1) {\
    count = count + 1;\
    }num = num + 1;}\
    return count;}\
    var total = this.count_primes(100);\
    system.println(total);",expect![[r#"
        > 25
    "#]]);
}

/// 递归式斐波那契
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

/// 循环式斐波那契
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
