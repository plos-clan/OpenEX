use std::{fmt::Display, str::FromStr};

pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown lint name `{}`", self.0)
    }
}

macro_rules! declare_lint {
    ($($name:ident $opt:literal),+ $(,)?) => {
        #[derive(Debug, Eq, PartialEq, Hash, Clone, Copy, Ord, PartialOrd)]
        pub enum Lint {
            $($name),+
        }

        impl FromStr for Lint {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($opt => Ok(Self::$name),)+
                    _ => Err(Error(s.into())),
                }
            }
        }
    };
}

declare_lint! {
    All "all",
    FuncNoArg "func-no-arg",
    LoopNoExpr "loop-no-expr",
    NoTypeGuess "no-type-guess",
    UnusedValue "unused-value",
    UnusedLibrary "unused-library",
}
