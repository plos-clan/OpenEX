pub mod compiler;
pub mod library;
pub mod runtime;

use crate::compiler::file::SourceFile;
use crate::compiler::Compiler;
use crate::library::load_libraries;
use crate::runtime::{MetadataUnit, MethodInfo};
use smol_str::{SmolStr, ToSmolStr};
use std::collections::HashSet;
use std::ffi::{c_char, CStr};
use std::ptr;
use crate::compiler::ast::vm_ir::Value;
use crate::runtime::thread::ThreadManager;

pub struct OpenEX {
    compiler: Compiler,
    metadata: Vec<MetadataUnit<'static>>,
}

#[repr(C)]
pub enum OpenExStatus {
    Success = 0,      // 成功
    ParseError = 2,   // 编译错误
    RuntimeError = 3, // 运行时错误
    FfiError = 4,     // ffi 交互异常
}

fn leak_string(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[unsafe(no_mangle)]
/// 对其他语言提供的本地接口, 该函数负责初始化编译前端环境并编译 `OpenEX` 标准库.
/// # Safety
/// `lib_path` 是一个有效的 C 字符串指针(以 NUL 结尾) 其字符串内容为标准库的路径
/// `lib_path` 可以为 NULL, 为NULL时解释器会在当前目录查找标准库实现
///
/// 函数返回值不应该被调用者修改内部的任何字段, 仅当句柄传递给各功能函数
/// C 函数定义: `void *openex_init()`;
pub unsafe extern "C" fn openex_init(lib_path: *const c_char) -> *mut OpenEX {
    let path = if lib_path.is_null() {
        None
    } else {
        let c_str = unsafe { CStr::from_ptr(lib_path) };
        match c_str.to_str() {
            Ok(rust_str) => Some(SmolStr::new(rust_str)),
            Err(_) => return ptr::null_mut(),
        }
    };

    let mut compiler = Compiler::new();
    match load_libraries(&mut compiler, path, &HashSet::new()) {
        Ok(()) => {}
        Err(_e) => {
            println!("error loading libraries");
            return ptr::null_mut();
        }
    }

    Box::into_raw(Box::new(OpenEX {
        compiler,
        metadata: Vec::new(),
    }))
}

#[unsafe(no_mangle)]
/// 往当前编译器句柄添加一个源文件
/// # Safety
/// # Panics
pub unsafe extern "C" fn openex_add_file(
    handle_raw: *mut OpenEX,
    source: *const c_char,
    name: *const c_char,
) -> OpenExStatus {
    if name.is_null() || source.is_null() {
        return OpenExStatus::FfiError;
    }
    let c_str_source = unsafe { CStr::from_ptr(source) };
    let r_source = match c_str_source.to_str() {
        Ok(rust_str) => Some(String::from(rust_str)),
        Err(_) => return OpenExStatus::FfiError,
    };

    let c_str_name = unsafe { CStr::from_ptr(name) };
    let r_name = match c_str_name.to_str() {
        Ok(rust_str) => Some(String::from(rust_str)),
        Err(_) => return OpenExStatus::FfiError,
    };

    if let Some(handle) = unsafe { handle_raw.as_mut() } {
        handle.compiler.add_file(SourceFile::new(
            r_name.unwrap(),
            r_source.unwrap(),
            HashSet::new(),
            false,
        ));
        OpenExStatus::Success
    } else {
        OpenExStatus::FfiError
    }
}

#[unsafe(no_mangle)]
/// 开始编译所有已经添加的源文件
/// # Safety
/// # Panics
/// 返回编译状态
pub unsafe extern "C" fn openex_compile(handle_raw: *mut OpenEX) -> OpenExStatus {
    if let Some(handle) = unsafe { handle_raw.as_mut() } {
        match handle.compiler.compile() {
            Ok(()) => OpenExStatus::Success,
            Err(()) => OpenExStatus::ParseError,
        }
    } else {
        OpenExStatus::FfiError
    }
}

#[unsafe(no_mangle)]
/// 初始化执行引擎环境, 必须编译完所有文件后再调用此函数, 否则新加入的函数无法被添加到执行引擎环境
/// 此函数只能调用一次, 多次调用会导致未定义行为
/// 在调用此函数前不得运行执行引擎
/// # Safety
/// # Panics
pub unsafe extern "C" fn openex_initialize_executor(handle_raw: *mut OpenEX) -> OpenExStatus {
    if let Some(handle) = unsafe { handle_raw.as_mut() } {
        let mut metadata: Vec<MetadataUnit<'static>> = Vec::new();
        for file in handle.compiler.get_files() {
            let vm_ir = file.ir_table.as_ref().unwrap();
            let mut methods: Vec<MethodInfo> = vec![];

            for func in vm_ir.get_functions() {
                methods.push(MethodInfo {
                    name: func.name.clone(),
                    r_name: func.filename.split('.').next().unwrap().to_smolstr(),
                    locals: func.locals,
                    codes: func.clone_codes().unwrap_or_default(),
                    is_native: func.is_native,
                    args: func.args,
                });
            }

            //WARN ffi 主动解控元数据单元名称的内存管理
            let name_owned = file.name.split('.').next().unwrap().to_string();
            let static_name: &'static str = leak_string(name_owned);

            metadata.push(MetadataUnit {
                //WARN fii 主动解控常量表内存管理
                constant_table: Box::leak(Box::new(vm_ir.get_constant_table())),
                names: static_name,
                methods,
                globals: vm_ir.get_locals_len(),
                root_code: vm_ir.clone_codes(),
                library: file.is_library,
            });
        }

        handle.metadata = metadata;
        OpenExStatus::Success
    } else {
        OpenExStatus::FfiError
    }
}

#[unsafe(no_mangle)]
/// 调用一个 `OpenEX` 函数, 需要指定文件名和函数名
/// # Safety
/// # Panics
pub unsafe extern "C" fn openex_call_function(
    handle_raw: *mut OpenEX,
    file: *const c_char,
    func: *const c_char,
) -> OpenExStatus{
    if file.is_null() || func.is_null() {
        return OpenExStatus::FfiError;
    }
    let c_str_file = unsafe { CStr::from_ptr(file) };
    let r_file = match c_str_file.to_str() {
        Ok(rust_str) => Some(String::from(rust_str)),
        Err(_) => return OpenExStatus::FfiError,
    }.unwrap();

    let c_str_func = unsafe { CStr::from_ptr(func) };
    let r_func = match c_str_func.to_str() {
        Ok(rust_str) => Some(String::from(rust_str)),
        Err(_) => return OpenExStatus::FfiError,
    }.unwrap();

    if let Some(handle) = unsafe { handle_raw.as_mut() } {
        let main_metadata = {
            let mut ret_file = None;
            for file in &handle.metadata {
                if file.names == r_file.as_str() {
                    ret_file = Some(file);
                    break
                }
            }
            match ret_file {
                Some(file) => file,
                None => return OpenExStatus::RuntimeError,
            }
        };

        let main_method = {
            let mut ret_func = None;
            for funcs in &main_metadata.methods {
                if funcs.name.as_str() == r_func {
                    ret_func = Some(funcs);
                    break
                }
            }
            match ret_func {
                Some(func) => func,
                None => return OpenExStatus::RuntimeError,
            }
        };

        std::thread::scope(|scope| {
            let thread_manager = ThreadManager::new(scope);
            thread_manager.submit_join_thread(main_metadata, main_method, handle.metadata.as_slice());
        });

        OpenExStatus::Success
    }else {
        OpenExStatus::FfiError
    }
}

#[unsafe(no_mangle)]
/// 释放掉 `OpenEX` 句柄环境占用的资源
/// # Safety
/// # Panics
pub unsafe extern "C" fn openex_free(handle_raw: *mut OpenEX) -> OpenExStatus {
    if handle_raw.is_null() {
        return OpenExStatus::FfiError;
    }

    let handle = unsafe { Box::from_raw(handle_raw) };
    for unit in handle.metadata {
        unsafe {
            // 回收名称
            if !unit.names.is_empty() {
                let _ = Box::from_raw(unit.names as *const str as *mut str);
            }
            // 回收常量表
            if !unit.constant_table.is_empty() {
                let _ = Box::from_raw(unit.constant_table as *const [Value] as *mut [Value]);
            }
        }
    }

    OpenExStatus::Success
}