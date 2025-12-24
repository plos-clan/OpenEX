# 扩展交互接口 CFFI

用于向其他程序提供对 OpenEX 的控制接口, 使 OpenEX 作为一个小型脚本解释器嵌入进其他程序中.

> 该库并不是为 OpenEX 脚本源码提供的, 适用于作为动态链接库形式存在的 OpenEX 解释器 \
> 在接口的参数无明确说明可以为空的情况下, 接口参数不得为空, 否则返回 `FfiError`

::: warning 线程安全

所有的 `CFFI` 交互接口都不是线程安全的, 您需要自行保证调用的同步性.

:::

## 句柄数据结构

* 状态枚举, 用于返回 OpenEX 各 cffi 交互接口的处理状态.

```C
typedef enum {
    Success = 0, // 成功
    ParseError = 2, // 编译器编译错误
    RuntimeError = 3, // 解释器运行时异常
    FfiError = 4, // FFI 接口交互异常
} OpenExStatus;
```

<br/>

* OpenEX CFFI 交互接口值传递

```C
typedef enum {
    Int = 0,   // OpenEX 整型 (int64_t)
    Bool = 1,  // OpenEX 布尔型 (int ? 1 : 0)
    Float = 2, // OpenEX 浮点型 (double, float_64)
    String = 3,// OpenEX 字符串 (char*)
    Ref = 4,   // OpenEX 调用引用 (char*)
    Null = 5   // OpenEX 空值
} ValueTag;

typedef union {
    int64_t i;
    bool b;
    double f;
    const char* s;
} ValueData; // 因 OpenEX 是动态类型, 故会有统一交互结构体表示所有类型

typedef struct {
    ValueTag tag;
    ValueData data;
} CValue;
```

<br/>

* 交互句柄, 该句柄不应被修改内部任何字段, 仅递交给 OpenEX 交互接口.

```C
typedef struct OpenEX OpenEX;
```

## openex_init

用于初始化一个 OpenEX 前端编译器环境, 该函数调用过程会同时编译 OpenEX 标准库.

* `lib_path` - 指定 OpenEX 标准库文件夹.
> 为空时以当前程序工作目录为基准查找 lib 文件夹作为 OpenEX 标准库所在文件夹加载

* `return` - 返回一个交互实例句柄
> 返回值为空代表加载失败.

```c
OpenEX* openex_init(const char* lib_path);
```

## openex_add_file

向指定 OpenEX 交互实例递交一个源文件

* `handle` - 交互实例
* `code` - 源文件的文本形式源码, 支持 utf-8
* `name` - 源文件的文件名 (必须包含后缀名, 否则触发未定义行为)
* `return` - 返回源文件的添加状态
> 文件名用于表示后续的其他脚本以何种名称导入该脚本

```c
OpenExStatus openex_add_file(OpenEX* handle, const char* code, const char* name);
```

## openex_compile

启动指定 OpenEX 交互实例的编译.

* `handle` - 交互实例
* `return` - 会依照情况返回 `Success` `ParserError` `FfiError` 三种状态

```c
OpenExStatus openex_compile(OpenEX* handle);
```

## openex_initialize_executor

初始化解释器与启动执行引擎.

::: warning 注意

不得在解释器启动后再次调用 `openex_add_file` 和 `openex_compile` 函数, 再次调用添加的源文件不会被解释器识别.

以及本函数每一个交互实例只能调用一次, 多次调用会发生未定义行为.

:::

* `handle` - 交互实例
* `return` - 返回初始化的情况, 不为 `Success` 代表启动失败

```c
OpenExStatus openex_initialize_executor(OpenEX* handle);
```

## openex_call_function

调用指定脚本中的指定函数.

> 该函数执行 OpenEX 函数是同步的, 在脚本函数执行完毕前该函数不会返回 \
> 但是由脚本内部函数创建的异步线程不会影响该函数的返回.

* `handle` - 交互实例
* `file` - 调用的脚本名 (不需要后缀名, 以 `import` 语句导入名为准)
* `func` - 函数名
* `args_ptr` - 参数数组 (无参数传递该数组的数据为空但是指针不得为空)
* `arg_count` - 参数的个数
* `out_result` - 指针为空代表不接受返回值, 否则 OpenEX 会向该指针写入本次调用的返回值
> 指针内部数据需要清理, 因为当本次调用的函数没有主动返回值时, 该指针不会被写入任何数据
* `return` - 会依照情况返回 `Success` `RuntimeError` `FfiError` 三种状态

```c
OpenExStatus openex_call_function(OpenEX* handle, const char* file, const char* func, CValue* args_ptr, size_t arg_count, CValue* out_result);
```

## openex_free_c_value

释放掉 OpenEX 值传递句柄的占用.

> 该函数用于释放掉句柄内部包装的字符串/引用等对象内存占用, 不会释放句柄本身

* `c_val` - 占用的句柄
* `return` - 释放的结果状态

```c
OpenExStatus openex_free_c_value(CValue* c_val);
```

::: warning 不安全的悬垂指针

如果解释器正在使用这个句柄对象 (比如 call_function 正在运行或解释器其他线程正在使用该句柄) \
调用该函数进行释放会导致解释器内部发生悬垂引用, 最终导致整个应用程序崩溃.

:::

## openex_free

释放 OpenEX 交互实例并终止解释器.

> 不建议在其他 `call_function` 运行的过程中调用该函数.

* `handle` - 交互句柄
* `return` - 终止后的状态, 不为 `Success` 代表解释器终止失败
> 终止失败默认解释器的所属的异步线程仍在运行

```c
OpenExStatus openex_free(OpenEX* handle);
```

::: tip 终止行为

调用 `openex` 函数时会发生以下行为
1. 该函数会首先终止所有由该交互实例创建的所有解释器异步线程.
2. 然后释放掉交互实例内部执行引擎与前端编译器占用的资源.
3. 最后会释放掉句柄本身.

:::
