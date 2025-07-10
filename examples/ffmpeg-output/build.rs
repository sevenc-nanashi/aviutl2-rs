fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rustc-link-arg=strmiids.lib");
    println!("cargo::rustc-link-arg=ole32.lib");
    println!("cargo::rustc-link-arg=shlwapi.lib");
    println!("cargo::rustc-link-arg=gdi32.lib");
    println!("cargo::rustc-link-arg=oleaut32.lib");
    println!("cargo::rustc-link-arg=vfw32.lib");
    println!("cargo::rustc-link-arg=mfuuid.lib");
    println!("cargo::rustc-link-arg=mfplat.lib");
}
