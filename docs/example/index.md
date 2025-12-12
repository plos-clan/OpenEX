# 第一个 OpenEX 程序

> 如果还没有配置好 OpenEX, 请前往 [简介](/started)

## Hello! World!

创建一个名为 `script.exf` 的空文件, 并在其添加以下代码

```js
import "system";

system.println("Hello! World!");
```

接下来在终端中运行以下命令可以看到输出

::: code-group

```shell [shell-input]
openex script.exf
```

```shell [shell-output]
Hello! World!
```

:::

## 关键字

以下列出了 `RustEdition` 的所有关键字, 与 `Pro` 版本有所差异

> `global` `local` 被彻底废除, 不在作为 OpenEX 保留字影响函数名和变量名定义.

|        关键字 | 说明          | 更多                              |
|-----------:|:------------|:--------------------------------|
|   `import` | 导入一个依赖库     | 替代了原 `include` 关键字, 并支持字符串字面量导入 |
|      `var` | 定义一个变量      | 替代了原 `value` 关键字                |
| `function` | 定义一个函数      |                                 |
|   `native` | 修饰一个本地函数    | `RustEdition` 新增的关键字            |
|       `if` | 定义一个判断语句    |                                 |
|     `elif` | 否则如果子判断语句声明 |                                 |
|     `else` | 否则子判断语句声明   |                                 |
|     `this` | 代表当前正在执行的脚本 |                                 |
|   `return` | 返回语句声明      |                                 |
|    `while` | 循环语句声明      |                                 |
|      `for` | 循环/迭代语句声明   |                                 |
|     `true` | 布尔值真        |                                 |
|    `false` | 布尔值假        |                                 |
|     `null` | 空值          |                                 |
|    `break` | 循环退出        |                                 |
| `continue` | 取消本次循环      |                                 |

## 注释

单行注释:
```js
// 这是一个单行注释
```

多行注释
```js
/*
这是一个多行注释
 */
```
