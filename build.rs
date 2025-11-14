fn main() {
    // Only compile resources on Windows
    #[cfg(windows)]
    {
        let _ = embed_resource::compile("resources/sorahk.rc", embed_resource::NONE);
    }
}
