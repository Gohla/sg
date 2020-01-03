use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

use shaderc::{Compiler, ShaderKind};

fn main() {
  let mut compiler = Compiler::new().unwrap();
  compile_shader(&mut compiler, ShaderKind::Vertex, "src/triangle.vert.glsl", "../../../target/shader/triangle.vert.spirv");
  compile_shader(&mut compiler, ShaderKind::Fragment, "src/triangle.frag.glsl", "../../../target/shader/triangle.frag.spirv");
}

fn compile_shader<S: Into<PathBuf>, D: Into<PathBuf>>(compiler: &mut Compiler, kind: ShaderKind, src_path: S, dst_path: D) {
  let src_path = src_path.into();
  let dst_path = dst_path.into();
  let source_text = {
    let mut reader = OpenOptions::new()
      .read(true)
      .open(&src_path)
      .unwrap_or_else(|e| panic!("Failed to create a reader for source file '{:?}': {:?}", src_path, e));
    let mut string = String::new();
    reader.read_to_string(&mut string)
      .unwrap_or_else(|e| panic!("Failed to read source file '{:?}' into a string: {:?}", src_path, e));
    println!("cargo:rerun-if-changed={:?}", src_path);
    string
  };
  let result = compiler.compile_into_spirv(
    &source_text,
    kind,
    src_path.file_name().unwrap().to_str().unwrap(),
    "main",
    None
  ).unwrap_or_else(|e| panic!("Failed to compile shader file '{:?}': {:?}", src_path, e));
  let mut writer = OpenOptions::new()
    .write(true)
    .create(true)
    .open(&dst_path)
    .unwrap_or_else(|e| panic!("Failed to create a writer for destination file '{:?}': {:?}", dst_path, e));
  writer.write(result.as_binary_u8())
    .unwrap_or_else(|e| panic!("Failed to write bytes to destination file '{:?}': {:?}", dst_path, e));
}
