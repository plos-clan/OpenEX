# 函数

函数代表一个执行的集合, 可以传入参数, 也可以返回参数.


## 定义

函数采用 `function` 关键字进行声明, 以下代码片段展示了如何定义一个函数.

```js
// 无参数
function example() {
    
}

// 有参数
function example_1(args1) {
    
}
```

::: tip 简化写法

对于无参数的函数, `RustEdition` 增加了一个简化写法.

无需显式编写形参的声明括号.

```js
function example_no_arg {
    
}
```

:::

## 调用

对于调用外部函数, 你需要先将对方脚本使用 `import` 关键字导入进来, \
然后按照 `导入名.函数名(参数)` 的方式去调用.

```js
import system;

system.println("Hello! World!");
```

调用本脚本内定义的函数, 则可以使用 `this` 关键字替代脚本名

```js
function example() {
    
}

this.example();
```

> 在 `openex` v0.0.2 版本 `17630d8` 修复型递交后, 可以使用 `example()` 的简写写法来调用本脚本定义的函数.

## 递归

OpenEX 支持函数的递归调用写法, \
并在 `RustEdition` 版本中, 死递归不会导致解释器调用栈溢出. \
直到操作系统内存耗尽, 解释器进程会被操作系统酌情终止.

```js
function loop(){
    this.loop();
}
this.loop();
```

> 在 `Pro` 之前的 Java 语言版本中, 调用栈最大递归深度为 510, \
> 超过这个深度解释器会抛出调用栈深度过大的异常.

## 本地函数

经过 `native` 关键字修饰的函数会变成本地函数, 这类函数的实现由解释器或解释器本地扩展进行实现.
OpenEX 源码不需要也不能对其指定实现.

```js
function native print(output);
function native exit(code);
```

定义本地函数时, 编译器会检查脚本名和函数名, 并从库管理器中查找本地函数对应的实现. \
如果没有找到实现, 则编译器报错.

::: code-group

```shell [error_info]
SyntaxError(example.exf-line: 1 column: 17): no native implement.
1    | function native no_impl_native();
                       ^
```

```shell [source]
function native no_impl_native();
```
:::

## 函数返回

在 OpenEX 中, 使用 `return` 关键字定义一个返回语句.

如:
```js
function name() {
    return 12;
}
```

> 也就是说 OpenEX 不存在没有返回值的函数 (除非本地函数实现) \
> 一些文档中标注的没有返回值的函数可以丢弃返回值

::: tip 简略写法

在 OpenEX 中, 返回语句写成如下写法

```js
function example() {
    return;
}
```

与

```js
function example() {
    return null;
}
```

等效

:::
