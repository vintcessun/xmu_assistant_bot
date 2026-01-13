use base64::{Engine as _, engine::general_purpose};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=qqdata/face");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("face_data.rs");

    let mut builder = phf_codegen::Map::new();
    let face_dir = Path::new("qqdata/face");

    if let Ok(entries) = fs::read_dir(face_dir) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().is_some_and(|e| e == "gif") {
                let face_id = path.file_stem().unwrap().to_str().unwrap().to_string();

                // 读取原始字节
                let bytes = fs::read(&path).unwrap();
                // 转换为 Base64 字符串
                let b64_content = general_purpose::STANDARD.encode(bytes);

                // 构造存储三元组的 Rust 表达式字符串
                // 格式为: ("image/gif", "BASE64_DATA...", Some("NAME"))
                let val_expr = format!(
                    "(r#\"image/gif\"#, r#\"{}\"# , r#\"{}.gif\"#)",
                    b64_content, face_id
                );

                builder.entry(face_id, val_expr);
            }
        }
    }

    let code = format!(
        "static FACES: phf::Map<&'static str, (&'static str, &'static str, &'static str)> = {};\n",
        builder.build()
    );

    fs::write(dest_path, code).unwrap();
}
