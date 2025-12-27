# 依赖导入

OpenEX 采用 `import` 关键字来定义一个依赖导入语句.

导入名以源文件的名称为准, 且导入名会影响变量定义, 变量名不允许与导入名重复.

## 旧版写法

```js
import "library_name";
import library_name;
```

直接在后方导入依赖库的名称即可.

> 旧版写法会长期支持下去, 不会被废弃

## 新版写法

在 `RustEdition v0.0.2` 版本后支持了如下写法

```js
import name from "library_name";
```

这种写法适用于源文件名无法被 `OpenEX` 识别成标识符的情况.
