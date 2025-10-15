fn main() -> anyhow::Result<()> {
    let out_dir = std::env::var("OUT_DIR")?;
    let hrir_dir = std::path::Path::new(&out_dir).join("hrir_data");
    fs_err::create_dir_all(&hrir_dir)?;
    let hrir_path = hrir_dir.join("hrir.bin");
    if !hrir_path.exists() {
        let tmp_path = hrir_path.with_extension("tmp");
        let download_url = "https://github.com/mrDIMAS/hrir_sphere_builder/raw/refs/heads/master/hrtf_base/IRCAM/IRC_1059_C.bin";
        let response = ureq::get(download_url).call()?;
        let mut out_file = std::fs::File::create(&tmp_path)?;
        std::io::copy(&mut response.into_body().into_reader(), &mut out_file)?;
        drop(out_file);
        fs_err::rename(&tmp_path, &hrir_path)?;
    }

    println!("cargo::rustc-env=HRIR_PATH={}", hrir_path.display());
    Ok(())
}
