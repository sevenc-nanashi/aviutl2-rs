use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let root_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let font_dir = root_dir.join("fonts");
    if font_dir.exists() {
        return Ok(());
    }
    std::fs::create_dir_all(&font_dir)?;

    download(
        "https://github.com/coz-m/MPLUS_FONTS/raw/refs/heads/master/fonts/otf/Mplus1-Regular.otf",
        &font_dir.join("Mplus1-Regular.otf"),
    )?;
    download(
        "https://github.com/coz-m/MPLUS_FONTS/raw/refs/heads/master/fonts/otf/Mplus1Code-Medium.otf",
        &font_dir.join("Mplus1Code-Medium.otf"),
    )?;

    println!("cargo::rerun-if-changed=build.rs");

    Ok(())
}

fn download(url: &str, out_path: &PathBuf) -> anyhow::Result<()> {
    let temp_path = out_path.with_extension("tmp");
    let response = ureq::get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("Failed to download {}: {}", url, e))?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download {}: HTTP {}",
            url,
            response.status()
        ));
    }
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| anyhow::anyhow!("Failed to create file {}: {}", out_path.display(), e))?;
    std::io::copy(&mut response.into_body().into_reader(), &mut file)
        .map_err(|e| anyhow::anyhow!("Failed to write to file {}: {}", out_path.display(), e))?;
    drop(file);
    std::fs::rename(&temp_path, out_path)
        .map_err(|e| anyhow::anyhow!("Failed to rename file {}: {}", temp_path.display(), e))?;
    Ok(())
}
