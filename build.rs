// Bake both maps on the build host when the `prebaked` feature is enabled.
// ARM clients only need immutable packet buffers in that mode.
#![allow(dead_code)]
#[cfg(feature = "prebaked")]
#[path = "src/colors.rs"]
mod colors;
#[cfg(feature = "prebaked")]
#[path = "src/course.rs"]
mod course;
#[cfg(feature = "prebaked")]
#[path = "src/layout.rs"]
mod layout;
#[cfg(feature = "prebaked")]
#[path = "src/mapgen.rs"]
mod mapgen;
#[cfg(feature = "prebaked")]
#[path = "src/materials.rs"]
mod materials;
#[cfg(feature = "prebaked")]
#[path = "src/wire.rs"]
mod wire;

fn main() {
    #[cfg(feature = "prebaked")]
    {
        use std::{env, fs, path::PathBuf};
        for f in [
            "assets/world.frames",
            "src/layout.rs",
            "src/colors.rs",
            "src/materials.rs",
            "src/wire.rs",
            "src/course.rs",
            "src/mapgen.rs",
            "build.rs",
        ] {
            println!("cargo:rerun-if-changed={f}")
        }
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        fs::write(out_dir.join("lobby.bin"), mapgen::build_world_frames(false)).unwrap();
        fs::write(out_dir.join("rally.bin"), mapgen::build_world_frames(true)).unwrap();
    }
}
