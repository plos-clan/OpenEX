#[cfg(test)]
thread_local! {
    static OUTPUT: std::cell::RefCell<Option<String>> = Default::default();
}

#[inline]
pub fn print(args: std::fmt::Arguments<'_>) {
    #[cfg(test)]
    OUTPUT.with_borrow_mut(|out| {
        if let Some(buf) = out {
            std::fmt::Write::write_fmt(buf, args).unwrap();
        } else {
            print!("{args}");
        }
    });
    #[cfg(not(test))]
    print!("{args}");
}

#[cfg(test)]
pub fn capture_output(f: impl FnOnce()) -> String {
    let old_output = OUTPUT.replace(Some(String::new()));

    f();

    OUTPUT.replace(old_output).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture() {
        let output = capture_output(|| {
            print(format_args!("foo"));
            print(format_args!("bar"));
        });
        assert_eq!(output, "foobar");
    }
}
