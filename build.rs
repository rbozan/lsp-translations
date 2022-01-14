use std::path::PathBuf;

fn main() {
    build_json();
    build_yaml();
    build_php();
}

fn build_json() {
    let json_dir: PathBuf = ["tree-sitter", "tree-sitter-json", "src"].iter().collect();

    cc::Build::new()
        .include(&json_dir)
        .file(json_dir.join("parser.c"))
        .flag_if_supported("-O")
        .compile("tree-sitter-json");
}

fn build_yaml() {
    let yaml_dir: PathBuf = ["tree-sitter", "tree-sitter-yaml", "src"].iter().collect();

    cc::Build::new()
        .include(&yaml_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-trigraphs")
        .flag_if_supported("-O")
        .file(yaml_dir.join("parser.c"))
        .compile("tree-sitter-yaml");

    cc::Build::new()
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

    cc::Build::new()
        .include(&php_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-trigraphs")
        .flag_if_supported("-O")
        .file(php_dir.join("parser.c"))
        .compile("tree-sitter-php");

    cc::Build::new()
        .cpp(true)
        .include(&php_dir)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-O")
        .file(php_dir.join("scanner.cc"))
        .compile("tree-sitter-php-scanner");
}
