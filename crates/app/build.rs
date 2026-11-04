//! MC-086, MC-087: rasterise the app icon for the window and the exe.
//!
//! `assets/icon.svg` is the artwork, byte for byte the SVG in
//! `docs/wiki/design/icon.md`, "The artwork". This renders it at each size in
//! `icon.md`, "Rasters" - every one from the SVG at that size, none shrunk
//! from another - and writes, into `OUT_DIR`:
//!
//! - `icon-256.rgba`, the runtime window icon (MC-086), which `gui::viewport`
//!   includes with `include_bytes!`. Raw pixels, row-major, 4 bytes each in
//!   R G B A order, with **straight** (unmultiplied) alpha, which is what
//!   `egui::IconData::rgba` means.
//! - `icon.ico`, the five sizes as PNG entries, and `icon.rc`, one `ICON`
//!   statement naming it, which `embed-resource` compiles with the Windows
//!   SDK's `rc.exe` and links into the `manhwa-cropper` bin (MC-087), so
//!   Explorer shows the icon before the app ever runs.
//!
//! Nothing rasterised is committed.
//!
//! tiny-skia's `Pixmap` holds premultiplied colour. Both outputs demultiply it
//! with the same per-pixel `PremultipliedColorU8::demultiply`: the raw file
//! through `Pixmap::take_demultiplied`, the PNGs through `Pixmap::encode_png`
//! (which calls `take_demultiplied` and encodes RGBA8). So the exe's 256 entry
//! decodes to exactly the bytes of `icon-256.rgba` (MC-087 AC-3).

use std::{env, fs, path::Path};

use resvg::{tiny_skia, usvg};

/// The runtime icon's edge, in pixels (`icon.md`, "Rasters").
const WINDOW_SIZE: u32 = 256;

/// The exe's icon sizes, in pixels (`icon.md`, "Rasters").
const EXE_SIZES: [u32; 5] = [16, 24, 32, 48, 256];

/// The bin the icon resource is linked into (`Cargo.toml`, `[[bin]]`).
const BIN: &str = "manhwa-cropper";

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.svg");
    println!("cargo:rerun-if-changed=build.rs");

    let svg = fs::read("assets/icon.svg").expect("read assets/icon.svg");
    let tree = usvg::Tree::from_data(&svg, &usvg::Options::default())
        .expect("assets/icon.svg is not a valid SVG");

    let out = env::var_os("OUT_DIR").expect("cargo sets OUT_DIR for build scripts");
    let out = Path::new(&out);

    let mut window = None;
    let pngs: Vec<(u32, Vec<u8>)> = EXE_SIZES
        .iter()
        .map(|&size| {
            let pixmap = render(&tree, size);
            let png = pixmap
                .encode_png()
                .unwrap_or_else(|e| panic!("encode the {size} px icon as PNG: {e}"));
            if size == WINDOW_SIZE {
                // One render is both the window icon and the exe's 256 entry.
                window = Some(pixmap);
            }
            (size, png)
        })
        .collect();

    let window = window.expect("the window icon's size is one of the exe's sizes");
    fs::write(out.join("icon-256.rgba"), window.take_demultiplied()).expect("write icon-256.rgba");
    fs::write(out.join("icon.ico"), ico(&pngs)).expect("write icon.ico");

    // One `ICON` statement and nothing else (MC-087, Out of scope). rc.exe
    // finds `icon.ico` beside the `.rc`; embed-resource also puts `OUT_DIR`
    // on its include path.
    let rc = out.join("icon.rc");
    fs::write(&rc, "1 ICON \"icon.ico\"\n").expect("write icon.rc");

    // `manifest_required`: on a Windows target, a missing `rc.exe` fails the
    // build rather than shipping an exe Explorer shows with the default icon.
    // Off Windows it does nothing and succeeds.
    embed_resource::compile_for(&rc, [BIN], embed_resource::NONE)
        .manifest_required()
        .unwrap_or_else(|e| panic!("embed the exe's icon resource: {e}"));
}

/// The SVG rendered at `size` x `size`. The viewBox is 64 units, so one unit
/// is `size / 64` px (`icon.md`, "Sample points").
fn render(tree: &usvg::Tree, size: u32) -> tiny_skia::Pixmap {
    let view = tree.size();
    let edge = size as f32; // exact: every size fits f32's mantissa
    let transform = tiny_skia::Transform::from_scale(edge / view.width(), edge / view.height());
    let mut pixmap =
        tiny_skia::Pixmap::new(size, size).unwrap_or_else(|| panic!("a {size} x {size} pixmap"));
    resvg::render(tree, transform, &mut pixmap.as_mut());
    pixmap
}

/// An `.ico` file whose entries are PNGs: a 6-byte `ICONDIR`, a 16-byte
/// `ICONDIRENTRY` per image, then the PNG bytes. `rc.exe` copies each entry's
/// width, height and bit count into the `RT_GROUP_ICON` it builds (observed in
/// MC-087: a bit count of 0 here reaches the exe as 0, which AC-1 refuses), so
/// they are written as the exe must carry them: 32 bpp, and a width or height
/// byte of 0 for 256. Planes is 1, as the format says.
fn ico(pngs: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = u16::try_from(pngs.len()).expect("a handful of icon sizes");
    let mut offset = 6 + 16 * u32::from(count);

    let mut file = Vec::new();
    file.extend_from_slice(&0u16.to_le_bytes()); // reserved
    file.extend_from_slice(&1u16.to_le_bytes()); // type: icon
    file.extend_from_slice(&count.to_le_bytes());

    for (size, png) in pngs {
        let edge = if *size == 256 {
            0
        } else {
            u8::try_from(*size).expect("icon sizes below 256 fit a byte")
        };
        let len = u32::try_from(png.len()).expect("an icon PNG under 4 GiB");
        file.push(edge); // width
        file.push(edge); // height
        file.push(0); // palette colours: none
        file.push(0); // reserved
        file.extend_from_slice(&1u16.to_le_bytes()); // planes
        file.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        file.extend_from_slice(&len.to_le_bytes());
        file.extend_from_slice(&offset.to_le_bytes());
        offset += len;
    }
    for (_, png) in pngs {
        file.extend_from_slice(png);
    }
    file
}
