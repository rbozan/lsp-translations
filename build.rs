use std::path::PathBuf;

fn main() {
    // println!("cargo:rustc-link-lib=static=stdc++");
    // println!("cargo:rustc-link-lib=static-nobundle=stdc++");

    build_json();
    build_yaml();
    build_php();
}

fn default_cc_builder() -> cc::Build {
    let mut build = cc::Build::new();
    build.shared_flag(true).static_flag(true);
    build
}

fn build_json() {
    let json_dir: PathBuf = ["tree-sitter", "tree-sitter-json", "src"].iter().collect();

    default_cc_builder()
        .include(&json_dir)
        .file(json_dir.join("parser.c"))
        .flag_if_supported("-O")
        .compile("tree-sitter-json");
}

fn build_yaml() {
    let yaml_dir: PathBuf = ["tree-sitter", "tree-sitter-yaml", "src"].iter().collect();

    default_cc_builder()
        .include(&yaml_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-trigraphs")
        .flag_if_supported("-O")
        .file(yaml_dir.join("parser.c"))
        .compile("tree-sitter-yaml");

    default_cc_builder()
        .cpp(true)
        .include(&yaml_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-O")
        .file(yaml_dir.join("scanner.cc"))
        .compile("tree-sitter-yaml-scanner");
}

fn build_php() {
    let php_dir: PathBuf = ["tree-sitter", "tree-sitter-php", "src"].iter().collect();

    default_cc_builder()
        .include(&php_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-trigraphs")
        .flag_if_supported("-O")
        .file(php_dir.join("parser.c"))
        .compile("tree-sitter-php");

    default_cc_builder()
        .cpp(true)
        .include(&php_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-O")
        .file(php_dir.join("scanner.cc"))
        .compile("tree-sitter-php-scanner");
}
