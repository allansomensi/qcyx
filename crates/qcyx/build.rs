fn main() {
    #[cfg(windows)]
    embed_icon();
}

#[cfg(windows)]
fn embed_icon() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let icon = std::path::Path::new(&manifest_dir).join("../../pkg/windows/Product.ico");

    winresource::WindowsResource::new()
        .set_icon(icon.to_str().expect("icon path contains invalid UTF-8"))
        .compile()
        .expect("failed to embed Windows icon resource");
}
