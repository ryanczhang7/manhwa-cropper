//! MC-086: rasterise the app icon for the window.
//!
//! `assets/icon.svg` is the artwork, byte for byte the SVG in
//! `docs/wiki/design/icon.md`, "The artwork". This renders it at 256 x 256,
//! the runtime window icon (`icon.md`, "Rasters"), into
//! `OUT_DIR/icon-256.rgba`, which `gui::viewport` includes with
//! `include_bytes!`. Nothing rasterised is committed.
//!
//! The file is raw pixels, row-major, 4 bytes each in R G B A order, with
//! **straight** (unmultiplied) alpha, which is what `egui::IconData::rgba`
//! means. tiny-skia's `Pixmap` holds premultiplied colour, so every pixel is
//! demultiplied on the way out; the tests cannot see the difference (every
//! sample point has alpha 0 or 255, where the two agree), so it is done here
//! deliberately, not left to them.

use std::{env, fs, path::Path};

use resvg::{tiny_skia, usvg};

/// The runtime icon's edge, in pixels (`icon.md`, "Rasters").
const SIZE: u32 = 256;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.svg");
    println!("cargo:rerun-if-changed=build.rs");

    let svg = fs::read("assets/icon.svg").expect("read assets/icon.svg");
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default())
        .expect("assets/icon.svg is not a valid SVG");

    // The viewBox is 64 units, so this is x4: one unit is 4 px (`icon.md`,
    // "Sample points").
    let size = tree.size();
    let edge = SIZE as f32; // exact: 256 fits f32's mantissa
    let transform = tiny_skia::Transform::from_scale(edge / size.width(), edge / size.height());

    let mut pixmap = tiny_skia::Pixmap::new(SIZE, SIZE).expect("a 256 x 256 pixmap");
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let rgba: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|premultiplied| {
            let c = premultiplied.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect();

    let out = env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build scripts");
    fs::write(Path::new(&out).join("icon-256.rgba"), rgba).expect("write icon-256.rgba");
}
