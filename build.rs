fn main() {
    println!("cargo:rerun-if-changed=lzrw3a/");

    cc::Build::new()
        .files(["lzrw3a/lib.c", "lzrw3a/lzrw3.c", "lzrw3a/lzrw3-a.c"])
        .include("lzrw3a")
        .compile("lzrw3a");
}
