# 命令行参数

## 帮助信息

* `-h` `--help` 参数可以获取命令行信息

```shell
openex --help
```

## 版本号

* `-v` `--version` 参数可以获取 OpenEX 当前的版本号和版权信息.

```shell
openex --version
```

## 命令行模式

* `--cli` 参数可以使 OpenEX 从 `stdin` 中获取输入并运行.

::: info 参数注意

`--cli` 命令没有简写语法, 不能写成如 `-c` 等参数简写.

:::

```shell
openex --cli
```

## 源文件输入

* 该参数无需任何参数前缀, 直接将文件名写在参数里即可, 并且支持输入多个源文件

```shell
openex your_script.exf your_script_2.exf
```

## 警告

* `-A` `--allow` 可以关闭 OpenEX 编译期输出的警告, 后续跟上需要关闭的条目

:::info 说明

`OpenEX` 编译器默认打开所有警告提示, 除非该参数明确指定关闭所有或具体条目的警告类型.

:::

```shell
openex -A<警告条目>
openex --allow=<警告条目>
```

|        条目        | 警告描述          |
|:----------------:|:--------------|
|      `all`       | 关闭所有警告信息      |
|  `func-no-arg`   | 关闭函数形参定义简写提示  |
|  `loop-no-expr`  | 关闭死循环语句定义简写提示 |
| `no-type-guess`  | 关闭类型推断失败提示    |
|  `unused-value`  | 关闭未使用的变量提示    |
| `unused-library` | 关闭未使用的库导入提示   |