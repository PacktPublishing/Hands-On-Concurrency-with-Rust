extern crate cc;

fn main() {
    cc::Build::new()
        .file("c_src/sim.c")
        .flag("-std=c11")
        .flag("-O3")
        .flag("-Wall")
        .flag("-Werror")
        .flag("-Wunused")
        .flag("-Wpedantic")
        .flag("-Wunreachable-code")
        .compile("mars");
}
