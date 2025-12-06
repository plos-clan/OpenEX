# OpenEX RustEdition

OpenEX是一个编译解释一体化的脚本语言.

继上一个 `Pro` 分支版本的新主要版本, 采用 Rust 语言重写.

## 协议

RustEdition 采用 [MIT License](LICENSE) 协议进行开源.

## 语法更改

相比于 `Pro` 版本, 部分关键字和语法结构有所更改.

* `local` 和 `global` 不再被作为保留字
* 变量定义由 `value` 关键字换成 `var` 关键字 (`value` 不作保留字)
* 库导入由 `include` 关键字换成 `import` 关键字 (`include` 不作保留字)
* 函数定义在无形参时可省略 `()` - `function name { /* body */ }`
* `while` 循环在表达式总是为真时可省略  `()` - `while { /*body*/ }`
