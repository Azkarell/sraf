use  std::{fs::*, path::Path};
fn main() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("test_app").join("shaders").join("shader.wgsl");
    shader_editor_lib::test(path);
}
