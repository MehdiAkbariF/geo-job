fn main() {
    // Recompile this crate whenever any migration script is added or modified
    println!("cargo:rerun-if-changed=../../migrations");
}