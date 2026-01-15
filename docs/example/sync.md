# 线程安全

OpenEX 支持多线程运行脚本与脚本内的方法, 
故解释器内会具备一些安全措施用于保证多线程环境下脚本的执行符合预期.

## 函数安全

OpenEX 支持使用 `sync` 关键字来修饰一个函数, 其会避免多个线程同时执行该函数.

```js
function sync safe_func() {
    // 函数实现
}
```

::: warning 递归问题

不建议在被 `sync` 关键字修饰的函数使用递归调用, 这会导致较为复杂的同步处理.

> 虽然 OpenEX 解释器支持 `sync` 函数重入

:::

::: danger 死锁问题

OpenEX 不会检查复杂的递归调用是否会发生潜在的死锁问题, 所以您应该避免以下写法.

```js
function sync example_sync_1() {
    this.example_sync_2();
}

function sync example_sync_2() {
    this.example_sync_1();
}
```

这会导致解释器当前线程直接发生死锁, 重入功能仅处理函数自递归调用.

:::

以及 `native` 关键字与 `sync` 关键字不能同时出现在一条函数定义上, \
有关于本地方法的线程安全措施由解释器或依赖库内部实现安全.
