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

~~实现过长, 请查阅标准库源码~~

> 采用简易牛顿迭代法实现 $x_{next} = \frac{1}{2}(x + \frac{n}{x})$

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

## `log` 对数运算

* 形参: `n` : 需要计算对数的正数 (类型限制: 必须为 number 或 float 类型)
* 返回值: 代表 $e ^ x$ 的近似值

~~实现过长, 请查阅标准库源码~~

> 采用牛顿迭代法实现 $x_{next} = x + \frac{n}{e^x} - 1$

## `cbrt` 指数函数

* 形参: `n` : 需要计算立方根的数值 (类型限制: 必须为 number 或 float 类型)
* 返回值: 代表 $\sqrt[3]{n}$ 的近似值

~~实现过长, 请查阅标准库源码~~

> 采用牛顿迭代法实现：$x_{next} = \frac{1}{3}(2x + \frac{n}{x^2})$


## `exp` 指数运算

* 形参: `x` : 指数幂 (类型限制: 必须为 number 或 float 类型)
* 返回值: 代表 $e^x$ 的近似值

~~实现过长, 请查阅标准库源码~~

> 采用泰勒级数展开实现：$e^x = \sum_{i=0}^{\infty} \frac{x^i}{i!}$

## `pow` 幂运算

* 形参: `base` : 底数； `exp_val` : 指数
* 返回值: 代表 $base^{exp\_val}$ 的计算结果

```js
function pow(base, exp_val) {
    if (base == 0) { return 0; }
    if (exp_val == 0) { return 1; }
    if (base == 0.0) { return 0.0; }
    if (exp_val == 0.0) { return 1.0; }
    // 上述 if 根据不同输入类型返回不同的值
    
    var res = this.exp(exp_val * this.log(base));
    return res;
}
```

> 采用对数恒等式实现：$base^{exp\_val} = e^{exp\_val \cdot \ln(base)}$
