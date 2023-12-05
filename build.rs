#[cfg(target_os = "windows")]
fn main() -> std::io::Result<()> {
    use winres::WindowsResource;
    use time::OffsetDateTime;

    WindowsResource::new()
        .set_icon("./bins/build.ico")
        .set("ProductName", "MongoBackuper")
        .set("OriginalFilename", "MongoBackuper.exe")
        .set("FileDescription", "Create backups of MongoDB")
        .set("LegalCopyright", format!("Copyright © fydne {}", OffsetDateTime::now_utc().year()).as_str())
        .set_manifest_file("./bins/installer_manifest.xml")
        .compile()
        .unwrap();

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() {
    // nothing, uses default cargo builder
}