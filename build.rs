use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");

    if !Path::new("assets/icon.ico").exists() {
        println!("cargo:warning=assets/icon.ico is missing so the binary has no icon");
    }

    if !Path::new("assets/sim.js").exists() {
        println!("cargo:warning=assets/sim.js is missing so raids cannot run");
    }
}
