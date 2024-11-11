fn main() {
    if cfg!(target_os = "windows") {
        let opencv_libs = if cfg!(debug_assertions) {
            vec!["opencv_world480d"]
        } else {
            vec!["opencv_world480"]
        };

        for lib in opencv_libs {
            println!("cargo:rustc-link-lib=dylib={}", lib);
        }

        println!("cargo:rustc-link-search=native=C:/tools/opencv/build/x64/vc16/lib");
        println!("cargo:rustc-link-search=native=C:/tools/opencv/build/x64/vc16/bin");
    }
}