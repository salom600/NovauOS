// build.rs — link against system libpam.
//
// On Debian/Ubuntu this is `libpam.so.0` shipped by the `libpam0g` package.
// We use pkg-config if available, otherwise fall back to a bare `-lpam`.
fn main() {
    match pkg_config::Config::new().probe("pam") {
        Ok(_) => {
            println!("cargo:rustc-link-lib=pam");
        }
        Err(e) => {
            println!("cargo:warning=pkg-config could not find pam ({e}); falling back to -lpam");
            println!("cargo:rustc-link-lib=pam");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}
