# 语言标准库 type

为了弥补 OpenEX 隐式类型定义而出现的标准库, 可用于类型转换和类型校验.

## `to_number` 整型转换

* 形参: `auto` : 被转换的变量或字面量 (类型限制: string 或 float)
* 返回值: 转换后的整型数字

> `to_number` 函数是一个本地方法, 由解释器进行实现.

## `to_float` 浮点转换

* 形参: `auto` : 被转换的变量或字面量 (类型限制: string 或 number)
* 返回值: 转换后的浮点数字

> `to_float` 函数是一个本地方法, 由解释器进行实现.

## `check_type` 检查类型

* 形参: `auto` : 任何类型
* 返回值: 以字符串形式的类型名

> `check_type` 函数是一个本地方法, 由解释器进行实现.

## `to_bool` 布尔转换

* 形参: `auto` : 被转换的变量或字面量 (类型限制: string 或 number)
* 返回值: 不为数字 `0` 或字符串 `false` 一律返回 `false`

```js
function to_bool(auto) {
    return auto == 0 || auto == "true";
}
```

## `to_string` 字符串转换

* 形参: `auto` : 任何类型
* 返回值: 转换后的字符串

```js
function to_string(auto) {
    return auto + "";
}
```
