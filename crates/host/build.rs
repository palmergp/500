// Gives the Windows program the game's icon, and rebuilds it when the embedded web app changes.

fn main() {
    println!("cargo:rerun-if-changed=assets/500.ico");
    if std::env::var_os("CARGO_FEATURE_EMBED").is_some() {
        println!("cargo:rerun-if-changed=../../web/dist");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/500.ico");
        resource.set("FileDescription", "500");
        resource.set("ProductName", "500");
        if let Err(e) = resource.compile() {
            println!("cargo:warning=the program will have no icon: {e}");
        }
    }
}
