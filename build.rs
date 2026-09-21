fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out = std::env::var("OUT_DIR").unwrap();
    let wrap = std::path::PathBuf::from(&out).join("sbf-link");
    let real = std::env::var("CARGO_TARGET_SBPF_SOLANA_SOLANA_LINKER")
        .or_else(|_| std::env::var("CARGO_TARGET_SBF_SOLANA_SOLANA_LINKER"))
        .unwrap_or_else(|_| "rust-lld".to_string());
    let script = format!(
        "#!/bin/sh\nset -e\nLLD=\"{}\"\n\"$LLD\" \"$@\"\nOUT=\"\"\nprev=\"\"\nfor a in \"$@\"; do\n  if [ \"$prev\" = \"-o\" ]; then OUT=\"$a\"; fi\n  prev=\"$a\"\ndone\nif [ -n \"$OUT\" ] && [ -f \"$OUT\" ]; then\n  MAG=$(dd if=\"$OUT\" bs=1 count=4 2>/dev/null || true)\n  if [ \"$MAG\" = \"$(printf '\\177ELF')\" ]; then\n    printf '\\000' | dd of=\"$OUT\" bs=1 seek=7 conv=notrunc 2>/dev/null || true\n  fi\nfi\n",
        real.replace('\\', "\\\\").replace('"', "\\\"")
    );
    std::fs::write(&wrap, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrap, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    println!("cargo:rustc-linker={}", wrap.display());
}
