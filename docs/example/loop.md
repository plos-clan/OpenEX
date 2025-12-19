# 循环语句

## while 循环

一个普通的 `while` 循环语句写法如下

```js
while (布尔表达式) {
    
}
```

::: tip 简化写法

对于死循环, `RustEdition` 增加了一个简化写法.
无需显式指定布尔表达式定义

```js
while {
    
}
```

> 在编译器推断出 while 中表达式的值恒为 `true` 时, 会发出此简化写法的建议性警告.

:::

## IR层

对于之前的 OpenEX 版本, `RustEdition` 则会将循环翻译成 `JumpTrue` 和 `Jump` 字节码.

> 因为 `RustEdition` 的 IR 不再是具有树形结构的执行节点, 而是直接处理成扁平化的字节码. \
> 故添加了条件跳转指令和无条件跳转替换了 Java 中的循环执行节点.
