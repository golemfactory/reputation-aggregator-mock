use static_files::NpmBuild;

fn main() -> std::io::Result<()> {
    // trigger recompilation when a new migration is added
    println!("cargo:rerun-if-changed=migrations");
    NpmBuild::new("web")
        .install()?
        .run("build")?
        .target("web/dist/bundle")
        .change_detection()
        .to_resource_dir()
        .build()?;
    Ok(())
}
