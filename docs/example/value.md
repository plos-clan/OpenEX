# 变量

OpenEX 变量用于临时存储数据, 在运行过程中可以更改，获取变量内部的值.

## 定义

采用以下语句形式可以定义一个变量

```js
var name = 123;
```

同时如果变量的初始值为 `null` 可以简化写法.

```js
var name = null;
var name_1; // 简化写法可以省略初始值
```

此时 `name_1` 变量会被编译器自动初始化成 `null`

## 数组变量

数组变量在 OpenEX 是一个特殊的变量类型, 其加载指令独立于其他变量.

以下定义一个数组变量.

```js
var arr = [1,2,3,4];
```

数组变量可以使用 `[<index>]` 获取, 和重赋值

> index 的类型必须为 `number` 其余类型传入会发生类型转换异常

```js
arr[1] = 14;
var b = arr[0];
```

OpenEX 中不要求数组中的元素类型相同, 一个数组可以存在多个不同类型的元素.

```js
var arr_type = [true, 1, 3, 4, null];
```

::: tip 简化写法

OpenEX 支持直接使用数组内的方法来直接获取数组长度 `array.length()`

```js
var arr_type = [true, 1, 3, 4, null];
var len = arr_type.length();
```

> 这种简化写法最终会被编译器翻译成 `type.array_length()` 的函数调用

:::

## 全局变量

在脚本根作用域定义的变量为全局变量, 在脚本根栈帧的变量表开辟空间.

```js {1}
var a = 123; // 全局变量

if (1 + 1 == 2) {
    var b = 444; // 局部变量
}

function example() {
    var local = 0; // 局部变量
}
```

OpenEX 不允许直接对非本脚本内的全局变量进行操作, 这将带来不可控的风险. \
如果要使其他脚本可以操作全局变量, 可以对操作方式进行封装.

::: code-group


```js [example_1.exf]
import sys from "system";
import exa from "example_2";

exa.set(12);

system.println(exa.get());

```

```js [example_2.exf]

var global_var = 0;

function set(auto) {
    // 可以在这里添加类型检查等措施.
    global_var = auto;
}

function get() {
    return global_var;
}

```

:::
