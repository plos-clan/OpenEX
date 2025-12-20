# 语言标准库 math

实现了 OpenEX 中常用的复杂数学操作,是OpenEX基础库之一.

可以使用以下代码在脚本中导入.

```js
import "math";
```

> `math` 中绝大部分的操作全部由 OpenEX 源码自行实现, ~~体现了OpenEX的健壮性~~

## `sqrt` 开平方运算

* 形参: `n` : 被开平方的数 (类型限制: 必须为 number 或 float 类型)
* 返回值: 开平方后的结果 (随传入参数类型)

```js{math.exf}
function sqrt(n) {
    /* 实现过长不展示 */
}
```

> 采用简易牛顿迭代法实现

## `abs` 绝对值运算

* 形参: `value` : 需要取绝对值的数 (类型限制: 必须为 number 或 float 类型)
* 返回值: 取绝对值的结果 (随传入参数类型)

```js{math.exf}
function abs(value) {
    if (value < 0) {
        return 0 - value;
    }
    return value;
}
```
