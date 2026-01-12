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

## for 循环 

一个普通的 `for` 循环写法如下

```js
for (变量定义;布尔表达式;表达式) {
    
}
```

OpenEX 支持您省略 `for` 语句的表达式组成部分

```js
var i = 0;
for (; i < 0;i++) {}
for (;;) {}
for(;;i++) {}
```
::: tip 循环判断特性

如果 `for` 语句的布尔表达式组成部分为空, 则该循环等效于 `while (true)`

但是只有所有的表达式组成部分为空, 编译器才会发出简化写法的建议性警告
> 如 `for(;;)`

:::


## 循环流程控制

> OpenEX 的循环流程控制语句与其他编程语言逻辑基本一致, 都是最近邻循环亲和性

* `continue` 语句可以取消本次循环的后续代码.

::: code-group

```js [source] {6}
var a = 0;
while (a < 10) {
    system.println("number: " + a);
    a++;
    if (a > 5) {
        continue;
    }
    system.println("a > 5");
}
```

```shell [output]
number: 0
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
```

:::


* `break` 语句会直接终止循环

::: code-group

```js [source] {7}
var a = 0;

while {
    system.println("number: " + a);
    a++;
    if (a > 5) {
        break;
    }
}
```

```js [output]
number: 0
number: 1
number: 2
number: 3
number: 4
number: 5
```

:::

## IR与语法树层

* `for` 循环和 `while` 循环最终都会被翻译成语法树的 `loop` 节点
* 对于之前的 OpenEX 版本, `RustEdition` 则会将循环翻译成 `JumpTrue` 和 `Jump` 字节码.

> 因为 `RustEdition` 的 IR 不再是具有树形结构的执行节点, 而是直接处理成扁平化的字节码. \
> 故添加了条件跳转指令和无条件跳转替换了 Java 中的循环执行节点.
