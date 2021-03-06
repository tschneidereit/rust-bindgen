use std::env;
use std::fs;
use std::path::Path;

const LINUX_CLANG_DIRS: &'static [&'static str] = &[
    "/usr/lib",
    "/usr/lib/llvm",
    "/usr/lib/llvm-3.4/lib",
    "/usr/lib/llvm-3.5/lib",
    "/usr/lib/llvm-3.6/lib",
    "/usr/lib64/llvm",
    "/usr/lib/x86_64-linux-gnu",
];
const MAC_CLANG_DIR: &'static str = "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib";
const WIN_CLANG_DIRS: &'static [&'static str] = &["C:\\Program Files\\LLVM\\bin", "C:\\Program Files\\LLVM\\lib"];

fn path_exists(path: &Path) -> bool {
    match fs::metadata(path) {
        Ok(_) => true,
        Err(_) => false
    }
}

fn main() {
    let use_static_lib = env::var_os("LIBCLANG_STATIC").is_some() || cfg!(feature = "static");

    let possible_clang_dirs = if let Ok(dir) = env::var("LIBCLANG_PATH") {
        vec![dir]
    } else if cfg!(any(target_os = "linux", target_os = "freebsd")) {
        LINUX_CLANG_DIRS.iter().map(ToString::to_string).collect()
    } else if cfg!(target_os = "macos") {
        vec![MAC_CLANG_DIR.to_string()]
    } else if cfg!(target_os = "windows") {
		WIN_CLANG_DIRS.iter().map(ToString::to_string).collect()
    } else {
        panic!("Platform not supported");
    };

    let clang_lib = if cfg!(target_os = "windows") {
			format!("libclang{}", env::consts::DLL_SUFFIX)
		} else {
			format!("{}clang{}", env::consts::DLL_PREFIX, env::consts::DLL_SUFFIX)
		};

    let maybe_clang_dir = possible_clang_dirs.iter().filter_map(|candidate_dir| {
        let clang_dir = Path::new(candidate_dir);
        let clang_path = clang_dir.join(clang_lib.clone());
        if path_exists(&*clang_path) {
            Some(clang_dir)
        } else {
            None
        }
    }).next();

    macro_rules! qw {
        ($($i:ident)*) => (vec!($(stringify!($i)),*));
    }

    if let Some(clang_dir) = maybe_clang_dir {
        if use_static_lib {
            let libs = qw![
                LLVMAnalysis
                LLVMBitReader
                LLVMCore
                LLVMLTO
                LLVMLinker
                LLVMMC
                LLVMMCParser
                LLVMObjCARCOpts
                LLVMObject
                LLVMOption
                LLVMScalarOpts
                LLVMSupport
                LLVMTarget
                LLVMTransformUtils
                LLVMVectorize
                LLVMipa
                LLVMipo
                clang
                clangARCMigrate
                clangAST
                clangASTMatchers
                clangAnalysis
                clangBasic
                clangDriver
                clangEdit
                clangFormat
                clangFrontend
                clangIndex
                clangLex
                clangParse
                clangRewrite
                clangRewriteFrontend
                clangSema
                clangSerialization
                clangStaticAnalyzerCheckers
                clangStaticAnalyzerCore
                clangStaticAnalyzerFrontend
                clangTooling
            ];

            print!("cargo:rustc-flags=");
            for lib in libs {
                print!("-l static={} ", lib);
            }
            println!("-L {} -l ncursesw -l z -l stdc++", clang_dir.to_str().unwrap());
        } else{
            println!("cargo:rustc-link-search=native={}", clang_dir.to_str().unwrap());
            println!("cargo:rustc-link-lib=dylib=clang");
        }
    } else {
        panic!("Unable to find {}", clang_lib);
    }
}
