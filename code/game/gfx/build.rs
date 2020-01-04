use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

use shaderc::{Compiler, ShaderKind};

fn main() {
  let mut compiler = Compiler::new().unwrap();
  let src_dir = Path::new("src");
  let dst_dir = Path::new("../../../target/shader");
  fs::create_dir_all(dst_dir)
    .unwrap_or_else(|e| panic!("Failed to create destination directory '{}': {:}", dst_dir.display(), e));
  compiler.compile_shader_pair(src_dir, dst_dir, "triangle");
}


trait CompilerEx {
  fn compile_shader<S: AsRef<Path>, D: AsRef<Path>>(&mut self, kind: ShaderKind, src_path: S, dst_path: D);
  fn compile_shader_pair<S: AsRef<Path>, D: AsRef<Path>>(&mut self, src_dir: S, dst_dir: D, name: &str);
}

impl CompilerEx for Compiler {
  fn compile_shader<S: AsRef<Path>, D: AsRef<Path>>(&mut self, kind: ShaderKind, src_path: S, dst_path: D) {
    let src_path = src_path.as_ref();
    let dst_path = dst_path.as_ref();
    let source_text = {
      let mut reader = OpenOptions::new()
        .read(true)
        .open(src_path)
        .unwrap_or_else(|e| panic!("Failed to create a reader for source file '{}': {:?}", src_path.display(), e));
      let mut string = String::new();
      reader.read_to_string(&mut string)
        .unwrap_or_else(|e| panic!("Failed to read source file '{}' into a string: {:?}", src_path.display(), e));
      println!("cargo:rerun-if-changed={}", src_path.display());
      string
    };
    let result = self.compile_into_spirv(
      &source_text,
      kind,
      src_path.file_name().map(|p| p.to_str().unwrap_or_default()).unwrap_or_default(),
      "main",
      None
    ).unwrap_or_else(|e| panic!("Failed to compile shader file '{}': {:?}", src_path.display(), e));
    let mut writer = OpenOptions::new()
      .write(true)
      .create(true)
      .open(dst_path)
      .unwrap_or_else(|e| panic!("Failed to create a writer for destination file '{}': {:?}", dst_path.display(), e));
    writer.write(result.as_binary_u8())
      .unwrap_or_else(|e| panic!("Failed to write bytes to destination file '{}': {:?}", dst_path.display(), e));
  }

  fn compile_shader_pair<S: AsRef<Path>, D: AsRef<Path>>(&mut self, src_dir: S, dst_dir: D, name: &str) {
    let src_dir = src_dir.as_ref();
    let dst_dir = dst_dir.as_ref();
    self.compile_shader(ShaderKind::Vertex, src_dir.join(format!("{}.vert.glsl", name)), dst_dir.join(format!("{}.vert.spv", name)));
    self.compile_shader(ShaderKind::Fragment, src_dir.join(format!("{}.frag.glsl", name)), dst_dir.join(format!("{}.frag.spv", name)));
  }
}
