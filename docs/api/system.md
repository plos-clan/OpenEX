# 语言标准库 system

实现了OpenEX的标准输入输出和线程创建、系统信息获取等函数
是OpenEX基础库之一,也是第一个被开发的运行库 \
可以使用以下代码在脚本中导入.

```js
import "system";
```

> 在 `Pro` 之前的版本中, 标准库中所有的接口全部由解释器进行实现. \
> `RustEdition` 将部分较为简单的封装函数移动到 OpenEX 源码中实现.

## `print` 标准输出

* 形参: `output` : 输出的信息
* 返回值: `NULL` : 该函数没有返回值
 
```js{system.exf}
function native print(output);
```

> `print` 函数是一个本地方法, 由解释器进行实现.

## `println` 行打印

* 形参: `output` : 输出的信息
* 返回值: `NULL` : 该函数没有返回值

```js
function println(output) {
    this.print(output + "\n");
}
```
