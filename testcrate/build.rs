use std::env;
use std::ffi::OsString;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if env::var("TARGET")
        .map(|target| target == "xtensa-esp32-espidf")
        .unwrap_or(false)
    {
        if let Ok(path) = env::var("DEP_ESP_IDF_EMBUILD_ENV_PATH") {
            let mut full_path = OsString::from(path);
            if let Some(existing) = env::var_os("PATH") {
                full_path.push(":");
                full_path.push(existing);
            }
            env::set_var("PATH", full_path);
        }

        embuild::build::LinkArgs::output_propagated("ESP_IDF")
            .expect("failed to propagate esp-idf linker args");
    }

    #[cfg(feature = "lua51")]
    let version = lua_src::Lua51;
    #[cfg(feature = "lua52")]
    let version = lua_src::Lua52;
    #[cfg(feature = "lua53")]
    let version = lua_src::Lua53;
    #[cfg(feature = "lua54")]
    let version = lua_src::Lua54;

    let artifacts = lua_src::Build::new().build(version);
    artifacts.print_cargo_metadata();
}
