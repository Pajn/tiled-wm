use std::env;
use std::path::PathBuf;

fn main() {
  // Tell cargo to tell rustc to link the system wlroots
  // shared library.
  println!("cargo:rustc-link-lib=wlroots");

  // let bindings = bindgen::Builder::default()
  //   // The input header we would like to generate
  //   // bindings for.
  //   .header("wlroots.h")
  //   // .opaque_type("const_pointer")
  //   // .opaque_type("std::vector")
  //   // .opaque_type("miral::.*")
  //   // .whitelist_type("wl_..*")
  //   // .whitelist_type("Mir.*")
  //   // .whitelist_type("mir::.*")
  //   // .whitelist_type("miral::.*")
  //   // .whitelist_function("wl_.*")
  //   // .whitelist_function("mir_.*")
  //   // .whitelist_function("miral::.*")
  //   // .no_copy("miral::.*")
  //   .default_enum_style(bindgen::EnumVariation::ModuleConsts)
  //   // .clang_args(vec!["-x", "c++"])
  //   .clang_args(vec!["-I", "/usr/include/pixman-1"])
  //   .clang_args(vec!["-I", "../subprojects/wlroots/build/protocol"])
  //   .clang_args(vec!["-I", "../subprojects/wlroots/include"])
  //   .clang_args(vec!["-DWLR_USE_UNSTABLE"])
  //   // Finish the builder and generate the bindings.
  //   .generate()
  //   // Unwrap the Result and panic on failure.
  //   .expect("Unable to generate bindings");

  // bindings
  //   .write_to_file("src/wlroots.rs")
  //   .expect("Couldn't write bindings!");
}
