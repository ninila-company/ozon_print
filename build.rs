fn main() {
    let ico_path = std::path::Path::new("assets/icon.ico");
    if ico_path.exists() {
        embed_resource::compile("assets/icon.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
