use std::fs;

fn main() {
    let path = "raylib-5.5_linux_amd64/lib";
    let path = fs::canonicalize(path).unwrap();

    println!(
        "cargo:rustc-link-search=native={}",
        path.as_path().display()
    );

    println!("cargo:rustc-link-lib=static=raylib");
}
