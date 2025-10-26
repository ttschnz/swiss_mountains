fn main() {
    println!("cargo:rustc-link-lib=spatialite");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}
