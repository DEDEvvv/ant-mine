fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap().replace('\\', "/");
    let cargo_dir = std::path::PathBuf::from(&manifest).join(".cargo");
    let _ = std::fs::create_dir_all(&cargo_dir);
    let wrap = cargo_dir.join("sbf-link");
    let script = format!(
        r#"#!/bin/sh
find_lld() {{
  for c in rust-lld sbpf-linker ld.lld; do
    if command -v "$c" >/dev/null 2>&1; then command -v "$c"; return; fi
  done
  find "$HOME" /root /opt /usr /solana /build -name ld.lld -type f 2>/dev/null | head -1
}}
LLD=$(find_lld)
if [ -z "$LLD" ]; then
  echo "sbf-link: cannot find lld" >&2
  exit 1
fi
"$LLD" "$@"
status=$?
if [ "$status" -ne 0 ]; then
  exit "$status"
fi
patch_elf() {{
  f="$1"
  [ -f "$f" ] || return 0
  mag=$(dd if="$f" bs=1 count=4 2>/dev/null || true)
  if [ "$mag" = "$(printf '\177ELF')" ]; then
    printf '\000' | dd of="$f" bs=1 seek=7 conv=notrunc
  fi
}}
scan_file() {{
  prev=""
  while IFS= read -r a || [ -n "$a" ]; do
    a=$(printf '%s' "$a" | tr -d '\r')
    if [ "$prev" = "-o" ]; then patch_elf "$a"; fi
    prev="$a"
  done < "$1"
}}
prev=""
for a in "$@"; do
  if [ "$prev" = "-o" ]; then patch_elf "$a"; fi
  case "$a" in
    @*)
      rf=${{a#@}}
      if [ -f "$rf" ]; then scan_file "$rf"; fi
      ;;
  esac
  prev="$a"
done
if [ -d "{manifest}/target" ]; then
  find "{manifest}/target" -name 'ant_mine.so' -type f 2>/dev/null | while read -r f; do
    patch_elf "$f"
  done
fi
exit 0
"#,
        manifest = manifest
    );
    std::fs::write(&wrap, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrap, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    println!("cargo:rustc-linker={}", wrap.display());
    println!("cargo:warning=ant-mine osabi wrapper {}", wrap.display());
}
