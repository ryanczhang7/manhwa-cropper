//! MC-087: the exe file itself carries the crop-corners icon, as a Windows
//! icon resource, for File Explorer, shortcuts and a pinned taskbar entry.
//!
//! # How the test sees the exe
//!
//! Cargo builds `manhwa-cropper.exe` before it runs this crate's integration
//! tests and hands its path over as `CARGO_BIN_EXE_manhwa-cropper`. The build
//! script runs for that build exactly as for release, so this file reads the
//! real resource. It parses the PE file itself - DOS header, PE signature,
//! COFF header, optional header, section table, then the resource directory
//! tree (type -> name/id -> language -> data) - because RED cannot add a
//! dependency. `image` decodes the PNG entries.
//!
//! # Where every number comes from
//!
//! The five sizes, 32 bpp and PNG entries are `docs/wiki/design/icon.md`,
//! "Rasters". The sample points are the story's AC-2: pixel (0,0), outside the
//! tile's rounded corner, and unit (8,32) of the 64-unit grid, on the tile
//! clear of panel and brackets. They are settled, not tuned: the tolerance of
//! 2 per channel at 16 and 24 px is the only allowance. The 256 entry must be
//! the window icon MC-086 ships (`gui::viewport().icon`), byte for byte.
//!
//! # How the checks prove they can tell good from bad
//!
//! Every check is a function over parsed data - the raw `RT_GROUP_ICON`
//! blobs, the group's entries, and a map from `RT_ICON` id to bytes - and
//! returns the checks that failed. The real exe must fail none. Each control
//! named in the acceptance criteria is built in memory in this file (a
//! `GRPICONDIR` blob, PNGs encoded with `image`) and run through the same
//! functions, asserting exactly which checks it fails; a positive control
//! must fail none, so a checker that rejects everything cannot pass.
//!
//! # Resource format, as opposed to an `.ico` file
//!
//! In the resource a `GRPICONDIR` is a 6-byte header (reserved 0, type 1,
//! count) and a 14-byte `GRPICONDIRENTRY` per image ending in the `u16` id of
//! its `RT_ICON` resource - not the 16-byte `ICONDIRENTRY` of an `.ico` file,
//! which ends in a `u32` file offset. `rc.exe` performs that conversion.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Cursor;
use std::sync::OnceLock;

use eframe::egui::IconData;
use image::{ImageFormat, RgbaImage};
use manhwa_cropper::gui;

const EXE: &str = env!("CARGO_BIN_EXE_manhwa-cropper");

// --- Settled values ----------------------------------------------------------

/// `RT_ICON`, one image.
const RT_ICON: u16 = 3;
/// `RT_GROUP_ICON`, the directory of images Explorer chooses from.
const RT_GROUP_ICON: u16 = 14;

/// The five sizes, px square (`icon.md`, "Rasters").
const SIZES: [u32; 5] = [16, 24, 32, 48, 256];
/// Every entry is 32-bit RGBA.
const BPP: u16 = 32;

/// `primary` light, `#0067C0`, the tile.
const TILE: [u8; 3] = [0x00, 0x67, 0xC0];

/// Unit (8,32) of the 64-unit grid at each size (AC-2), and the per-channel
/// tolerance allowed there: 2 at 16 and 24 px, exact at the others.
const TILE_SAMPLES: [(u32, (u32, u32), u8); 5] = [
    (16, (2, 8), 2),
    (24, (3, 12), 2),
    (32, (4, 16), 0),
    (48, (6, 24), 0),
    (256, (32, 128), 0),
];

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

// --- Failures ----------------------------------------------------------------

/// One failed check: which, and what it read.
#[derive(Debug)]
struct Failure {
    check: String,
    detail: String,
}

fn fail(check: impl Into<String>, detail: impl Into<String>) -> Failure {
    Failure {
        check: check.into(),
        detail: detail.into(),
    }
}

fn ids(failures: &[Failure]) -> BTreeSet<String> {
    failures.iter().map(|f| f.check.clone()).collect()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn report(failures: &[Failure]) -> String {
    failures
        .iter()
        .map(|f| format!("\n  {}: {}", f.check, f.detail))
        .collect()
}

// --- The PE resource parser --------------------------------------------------

/// A resource directory name: a numeric id or a UTF-16 string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ResId {
    Id(u16),
    Name(String),
}

/// One leaf of the resource tree.
#[derive(Debug, Clone)]
struct Resource {
    kind: ResId,
    name: ResId,
    lang: ResId,
    data: Vec<u8>,
}

/// What the parser found in a PE file.
#[derive(Debug)]
struct Resources {
    /// The name of the section holding the resource directory, if any.
    section: Option<String>,
    leaves: Vec<Resource>,
}

impl Resources {
    /// Every leaf as `type/name/language (bytes)`, for failure messages.
    fn listing(&self) -> Vec<String> {
        self.leaves
            .iter()
            .map(|r| {
                format!(
                    "{:?}/{:?}/{:?} ({} bytes)",
                    r.kind,
                    r.name,
                    r.lang,
                    r.data.len()
                )
            })
            .collect()
    }

    /// The resource types present, in order.
    fn kinds(&self) -> BTreeSet<ResId> {
        self.leaves.iter().map(|r| r.kind.clone()).collect()
    }

    fn of_kind(&self, kind: u16) -> impl Iterator<Item = &Resource> {
        self.leaves
            .iter()
            .filter(move |r| r.kind == ResId::Id(kind))
    }

    /// Every `RT_GROUP_ICON` blob.
    fn icon_groups(&self) -> Vec<&[u8]> {
        self.of_kind(RT_GROUP_ICON)
            .map(|r| r.data.as_slice())
            .collect()
    }

    /// Every `RT_ICON` by numeric id.
    fn icons(&self) -> BTreeMap<u16, Vec<u8>> {
        self.of_kind(RT_ICON)
            .filter_map(|r| match r.name {
                ResId::Id(id) => Some((id, r.data.clone())),
                ResId::Name(_) => None,
            })
            .collect()
    }
}

fn u16_at(b: &[u8], at: usize) -> Result<u16, String> {
    b.get(at..at + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| format!("u16 at {at:#x} is past the end ({} bytes)", b.len()))
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| format!("u32 at {at:#x} is past the end ({} bytes)", b.len()))
}

/// A section header's mapping from RVA to file offset.
struct Section {
    name: String,
    virtual_address: u32,
    virtual_size: u32,
    raw_size: u32,
    raw_pointer: u32,
}

impl Section {
    fn contains(&self, rva: u32) -> bool {
        let len = self.virtual_size.max(self.raw_size);
        rva >= self.virtual_address && rva - self.virtual_address < len
    }

    fn offset(&self, rva: u32) -> usize {
        (rva - self.virtual_address + self.raw_pointer) as usize
    }
}

/// Parse the resource tree of a PE (PE32 or PE32+) image.
fn parse_resources(pe: &[u8]) -> Result<Resources, String> {
    if pe.get(0..2) != Some(b"MZ") {
        return Err("no MZ signature: not a PE file".into());
    }
    let pe_at = u32_at(pe, 0x3C)? as usize;
    if pe.get(pe_at..pe_at + 4) != Some(b"PE\0\0") {
        return Err(format!("no PE signature at e_lfanew {pe_at:#x}"));
    }
    let coff = pe_at + 4;
    let section_count = u16_at(pe, coff + 2)? as usize;
    let optional_size = u16_at(pe, coff + 16)? as usize;
    let optional = coff + 20;
    let magic = u16_at(pe, optional)?;
    // The data directories follow the fixed fields: 96 bytes in PE32, 112 in
    // PE32+. Index 2 is the resource table.
    let directories = match magic {
        0x10B => optional + 96,
        0x20B => optional + 112,
        other => return Err(format!("unknown optional header magic {other:#x}")),
    };
    let directory_count = u32_at(pe, directories - 4)?;
    let (rsrc_rva, rsrc_size) = if directory_count > 2 {
        (
            u32_at(pe, directories + 2 * 8)?,
            u32_at(pe, directories + 2 * 8 + 4)?,
        )
    } else {
        (0, 0)
    };

    let table = optional + optional_size;
    let sections = (0..section_count)
        .map(|i| {
            let at = table + i * 40;
            let raw_name = pe
                .get(at..at + 8)
                .ok_or_else(|| format!("section header {i} is past the end"))?;
            let name = String::from_utf8_lossy(raw_name)
                .trim_end_matches('\0')
                .to_owned();
            Ok(Section {
                name,
                virtual_size: u32_at(pe, at + 8)?,
                virtual_address: u32_at(pe, at + 12)?,
                raw_size: u32_at(pe, at + 16)?,
                raw_pointer: u32_at(pe, at + 20)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    if rsrc_rva == 0 || rsrc_size == 0 {
        return Ok(Resources {
            section: None,
            leaves: Vec::new(),
        });
    }
    let rva_to_offset = |rva: u32| -> Result<usize, String> {
        sections
            .iter()
            .find(|s| s.contains(rva))
            .map(|s| s.offset(rva))
            .ok_or_else(|| format!("RVA {rva:#x} lies in no section"))
    };
    let home = sections
        .iter()
        .find(|s| s.contains(rsrc_rva))
        .ok_or_else(|| format!("resource RVA {rsrc_rva:#x} lies in no section"))?;
    let base = rva_to_offset(rsrc_rva)?;

    let mut leaves = Vec::new();
    walk(pe, base, base, &mut Vec::new(), &mut leaves, &rva_to_offset)?;
    Ok(Resources {
        section: Some(home.name.clone()),
        leaves,
    })
}

/// Walk one `IMAGE_RESOURCE_DIRECTORY` at file offset `dir`; `base` is the
/// file offset of the root, to which every offset in the tree is relative.
fn walk(
    pe: &[u8],
    base: usize,
    dir: usize,
    path: &mut Vec<ResId>,
    leaves: &mut Vec<Resource>,
    rva_to_offset: &dyn Fn(u32) -> Result<usize, String>,
) -> Result<(), String> {
    if path.len() >= 3 {
        return Err(format!("resource tree deeper than 3 levels at {dir:#x}"));
    }
    let named = u16_at(pe, dir + 12)? as usize;
    let numbered = u16_at(pe, dir + 14)? as usize;
    for i in 0..named + numbered {
        let entry = dir + 16 + i * 8;
        let name_field = u32_at(pe, entry)?;
        let target = u32_at(pe, entry + 4)?;
        let id = if name_field & 0x8000_0000 != 0 {
            let at = base + (name_field & 0x7FFF_FFFF) as usize;
            let len = u16_at(pe, at)? as usize;
            let units = (0..len)
                .map(|k| u16_at(pe, at + 2 + k * 2))
                .collect::<Result<Vec<_>, String>>()?;
            ResId::Name(String::from_utf16_lossy(&units))
        } else {
            ResId::Id(
                u16::try_from(name_field)
                    .map_err(|_| format!("resource id {name_field:#x} is not a u16"))?,
            )
        };
        path.push(id);
        if target & 0x8000_0000 != 0 {
            let sub = base + (target & 0x7FFF_FFFF) as usize;
            walk(pe, base, sub, path, leaves, rva_to_offset)?;
        } else {
            if path.len() != 3 {
                return Err(format!(
                    "data entry at depth {} (expected 3): {path:?}",
                    path.len()
                ));
            }
            let data_entry = base + target as usize;
            let rva = u32_at(pe, data_entry)?;
            let size = u32_at(pe, data_entry + 4)? as usize;
            let at = rva_to_offset(rva)?;
            let data = pe
                .get(at..at + size)
                .ok_or_else(|| format!("resource data {path:?} runs past the end"))?
                .to_vec();
            leaves.push(Resource {
                kind: path[0].clone(),
                name: path[1].clone(),
                lang: path[2].clone(),
                data,
            });
        }
        path.pop();
    }
    Ok(())
}

// --- The icon group ----------------------------------------------------------

/// One `GRPICONDIRENTRY`.
#[derive(Debug, Clone, Copy)]
struct GroupEntry {
    width: u8,
    height: u8,
    bit_count: u16,
    id: u16,
}

impl GroupEntry {
    /// The declared edge: in an icon group a byte of 0 means 256.
    fn declared(byte: u8) -> u32 {
        if byte == 0 { 256 } else { u32::from(byte) }
    }

    fn size(&self) -> (u32, u32) {
        (Self::declared(self.width), Self::declared(self.height))
    }
}

/// Parse a `GRPICONDIR`: reserved 0, type 1, count, then 14 bytes per entry
/// (width, height, colours, reserved, planes u16, bit count u16, bytes u32,
/// id u16).
fn parse_group(blob: &[u8]) -> Result<Vec<GroupEntry>, String> {
    let reserved = u16_at(blob, 0)?;
    let kind = u16_at(blob, 2)?;
    if reserved != 0 || kind != 1 {
        return Err(format!(
            "header reads reserved {reserved}, type {kind}; an icon group is 0, 1"
        ));
    }
    let count = u16_at(blob, 4)? as usize;
    (0..count)
        .map(|i| {
            let at = 6 + i * 14;
            if blob.len() < at + 14 {
                return Err(format!(
                    "entry {i} of {count} runs past the {}-byte group",
                    blob.len()
                ));
            }
            Ok(GroupEntry {
                width: blob[at],
                height: blob[at + 1],
                bit_count: u16_at(blob, at + 6)?,
                id: u16_at(blob, at + 12)?,
            })
        })
        .collect()
}

/// AC-1: exactly one icon group, listing exactly the five sizes, square,
/// 32 bpp.
fn group_failures(groups: &[&[u8]]) -> Vec<Failure> {
    if groups.len() != 1 {
        return vec![fail(
            "one icon group",
            format!("the resources hold {} RT_GROUP_ICON entries", groups.len()),
        )];
    }
    let entries = match parse_group(groups[0]) {
        Ok(entries) => entries,
        Err(e) => return vec![fail("group header", e)],
    };
    let mut failures = Vec::new();
    if entries.len() != SIZES.len() {
        failures.push(fail(
            "image count",
            format!(
                "the group lists {} images, expected {}",
                entries.len(),
                SIZES.len()
            ),
        ));
    }
    for want in SIZES {
        if !entries.iter().any(|e| e.size() == (want, want)) {
            failures.push(fail(
                format!("size {want} listed"),
                format!("no {want} x {want} image in the group"),
            ));
        }
    }
    for e in &entries {
        let (w, h) = e.size();
        if w != h || !SIZES.contains(&w) {
            failures.push(fail(
                format!("unexpected size (id {})", e.id),
                format!("the group lists a {w} x {h} image"),
            ));
        }
        if e.bit_count != BPP {
            failures.push(fail(
                format!("32 bpp @{w}"),
                format!(
                    "the {w} x {h} image (id {}) declares {} bpp",
                    e.id, e.bit_count
                ),
            ));
        }
    }
    failures
}

/// The PNG an entry points at, decoded to RGBA, or the failure that stops it.
fn decode(entry: &GroupEntry, icons: &BTreeMap<u16, Vec<u8>>) -> Result<RgbaImage, Failure> {
    let (w, _) = entry.size();
    let bytes = icons.get(&entry.id).ok_or_else(|| {
        fail(
            format!("RT_ICON present @{w}"),
            format!("the group points at RT_ICON {} and there is none", entry.id),
        )
    })?;
    if !bytes.starts_with(&PNG_SIGNATURE) {
        let head = &bytes[..bytes.len().min(8)];
        return Err(fail(
            format!("png @{w}"),
            format!(
                "RT_ICON {} starts {head:02X?}, not the PNG signature (a DIB?)",
                entry.id
            ),
        ));
    }
    image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map(|img| img.to_rgba8())
        .map_err(|e| {
            fail(
                format!("decodes @{w}"),
                format!("RT_ICON {}: {e}", entry.id),
            )
        })
}

/// AC-2, structure: every entry is a PNG whose decoded size is the size the
/// group declares for it.
fn structure_failures(entries: &[GroupEntry], icons: &BTreeMap<u16, Vec<u8>>) -> Vec<Failure> {
    entries
        .iter()
        .filter_map(|e| match decode(e, icons) {
            Err(f) => Some(f),
            Ok(img) => {
                let (w, h) = e.size();
                (img.dimensions() != (w, h)).then(|| {
                    fail(
                        format!("decoded size @{w}"),
                        format!(
                            "the group declares {w} x {h}, RT_ICON {} decodes to {} x {}",
                            e.id,
                            img.width(),
                            img.height()
                        ),
                    )
                })
            }
        })
        .collect()
}

/// AC-2, samples: at each of the five sizes, pixel (0,0) has alpha 0 and unit
/// (8,32) is `#0067C0` at alpha 255, RGB within the size's tolerance.
fn sample_failures(entries: &[GroupEntry], icons: &BTreeMap<u16, Vec<u8>>) -> Vec<Failure> {
    let mut failures = Vec::new();
    for (size, (x, y), tolerance) in TILE_SAMPLES {
        let img = entries
            .iter()
            .find(|e| e.size() == (size, size))
            .and_then(|e| decode(e, icons).ok())
            .filter(|img| img.dimensions() == (size, size));
        let Some(img) = img else {
            failures.push(fail(
                format!("sampled @{size}"),
                format!("no {size} x {size} PNG entry to sample"),
            ));
            continue;
        };
        let corner = img.get_pixel(0, 0).0;
        if corner[3] != 0 {
            failures.push(fail(
                format!("(0,0) alpha @{size}"),
                format!("expected alpha 0 outside the tile, read RGBA {corner:?}"),
            ));
        }
        let px = img.get_pixel(x, y).0;
        let ok = px[3] == 255
            && px[..3]
                .iter()
                .zip(TILE)
                .all(|(&have, want)| have.abs_diff(want) <= tolerance);
        if !ok {
            failures.push(fail(
                format!("tile ({x},{y}) @{size}"),
                format!(
                    "expected #0067C0 (each channel within {tolerance}) at alpha 255, read RGBA {px:?}"
                ),
            ));
        }
    }
    failures
}

/// AC-3: the 256 entry decodes to exactly the window icon's pixels.
fn identity_failures(
    entries: &[GroupEntry],
    icons: &BTreeMap<u16, Vec<u8>>,
    window: &IconData,
) -> Vec<Failure> {
    let Some(entry) = entries.iter().find(|e| e.size() == (256, 256)) else {
        return vec![fail("256 entry", "the group lists no 256 x 256 image")];
    };
    let img = match decode(entry, icons) {
        Ok(img) => img,
        Err(f) => return vec![f],
    };
    if img.dimensions() != (window.width, window.height) {
        return vec![fail(
            "same pixels as the window icon",
            format!(
                "the exe's 256 entry is {:?}, the window icon {} x {}",
                img.dimensions(),
                window.width,
                window.height
            ),
        )];
    }
    let exe = img.as_raw();
    if exe.as_slice() == window.rgba.as_slice() {
        return Vec::new();
    }
    let differing: Vec<usize> = exe
        .chunks(4)
        .zip(window.rgba.chunks(4))
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();
    let first = differing.first().map(|&i| {
        let (x, y) = (i as u32 % window.width, i as u32 / window.width);
        format!(
            "first at ({x},{y}): exe {:?}, window {:?}",
            &exe[i * 4..i * 4 + 4],
            &window.rgba[i * 4..i * 4 + 4]
        )
    });
    vec![fail(
        "same pixels as the window icon",
        format!(
            "{} of {} pixels differ (buffer lengths {} and {}); {}",
            differing.len(),
            window.width * window.height,
            exe.len(),
            window.rgba.len(),
            first.unwrap_or_else(|| "lengths differ".into())
        ),
    )]
}

// --- The real exe ------------------------------------------------------------

/// The built exe's resources, parsed once for the whole file.
fn exe_resources() -> &'static Resources {
    static PARSED: OnceLock<Resources> = OnceLock::new();
    PARSED.get_or_init(|| {
        let pe = std::fs::read(EXE).unwrap_or_else(|e| panic!("read {EXE}: {e}"));
        parse_resources(&pe).unwrap_or_else(|e| panic!("{EXE} is not a parseable PE: {e}"))
    })
}

/// The exe's one icon group, or a bug-report message saying why there is none.
fn exe_group() -> Vec<GroupEntry> {
    let res = exe_resources();
    let groups = res.icon_groups();
    assert!(
        groups.len() == 1,
        "the exe holds {} icon groups, not one: Explorer shows the default icon \
         (resource section {:?}, resource types present {:?})",
        groups.len(),
        res.section,
        res.kinds()
    );
    parse_group(groups[0]).unwrap_or_else(|e| panic!("the exe's icon group is malformed: {e}"))
}

fn window_icon() -> IconData {
    gui::viewport()
        .icon
        .as_deref()
        .expect("gui::viewport() carries no icon (MC-086)")
        .clone()
}

#[test]
fn the_exe_holds_one_icon_group_listing_16_24_32_48_and_256_at_32_bpp() {
    let res = exe_resources();
    let failures = group_failures(&res.icon_groups());
    assert!(
        failures.is_empty(),
        "AC-1: the exe's icon group is missing or wrong (resource section {:?}, resources {:?}):{}",
        res.section,
        res.listing(),
        report(&failures)
    );
}

#[test]
fn every_icon_image_in_the_exe_is_a_png_of_the_size_the_group_declares() {
    let entries = exe_group();
    let failures = structure_failures(&entries, &exe_resources().icons());
    assert!(
        failures.is_empty(),
        "AC-2: an icon image in the exe is not a PNG of its declared size:{}",
        report(&failures)
    );
}

#[test]
fn every_icon_image_in_the_exe_has_a_clear_corner_and_the_blue_tile() {
    let entries = exe_group();
    let failures = sample_failures(&entries, &exe_resources().icons());
    assert!(
        failures.is_empty(),
        "AC-2: an icon image in the exe is not the crop-corners artwork at its size:{}",
        report(&failures)
    );
}

#[test]
fn the_exes_256_image_is_the_window_icon_pixel_for_pixel() {
    let entries = exe_group();
    let failures = identity_failures(&entries, &exe_resources().icons(), &window_icon());
    assert!(
        failures.is_empty(),
        "AC-3: Explorer and the taskbar would show different 256 px icons:{}",
        report(&failures)
    );
}

// --- Controls, built in memory -----------------------------------------------

/// A `GRPICONDIR` blob listing `(edge, bpp, id)`, an edge of 256 written as 0.
fn group_blob(entries: &[(u32, u16, u16)]) -> Vec<u8> {
    let count = u16::try_from(entries.len()).expect("a small group");
    let mut blob = Vec::new();
    blob.extend(0u16.to_le_bytes());
    blob.extend(1u16.to_le_bytes());
    blob.extend(count.to_le_bytes());
    for &(edge, bpp, id) in entries {
        let byte = if edge == 256 {
            0
        } else {
            u8::try_from(edge).expect("an icon edge")
        };
        blob.extend([byte, byte, 0, 0]);
        blob.extend(1u16.to_le_bytes());
        blob.extend(bpp.to_le_bytes());
        blob.extend(0u32.to_le_bytes());
        blob.extend(id.to_le_bytes());
    }
    blob
}

fn png(img: &RgbaImage) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    img.write_to(&mut out, ImageFormat::Png)
        .expect("encode a PNG in memory");
    out.into_inner()
}

/// A solid opaque `#0067C0` square.
fn solid(edge: u32) -> RgbaImage {
    RgbaImage::from_pixel(edge, edge, image::Rgba([TILE[0], TILE[1], TILE[2], 255]))
}

/// The solid square with pixel (0,0) cleared: the least that passes AC-2.
fn passing(edge: u32) -> RgbaImage {
    let mut img = solid(edge);
    img.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
    img
}

/// The five sizes with ids 1..=5, and a PNG for each made by `draw`.
fn five(draw: impl Fn(u32) -> RgbaImage) -> (Vec<GroupEntry>, BTreeMap<u16, Vec<u8>>) {
    let listed: Vec<(u32, u16, u16)> = SIZES
        .iter()
        .zip(1u16..)
        .map(|(&edge, id)| (edge, BPP, id))
        .collect();
    let entries = parse_group(&group_blob(&listed)).expect("a well-formed control group");
    let icons = listed
        .iter()
        .map(|&(edge, _, id)| (id, png(&draw(edge))))
        .collect();
    (entries, icons)
}

/// Today's exe, as the parser measured it in RED, carries resources but no
/// icon group: a lone manifest-type entry stands in for that.
#[test]
fn resources_with_no_icon_group_fail_ac1_on_the_group_count() {
    let today = Resources {
        section: Some(".rsrc".into()),
        leaves: vec![Resource {
            kind: ResId::Id(24),
            name: ResId::Id(1),
            lang: ResId::Id(0x409),
            data: b"<assembly/>".to_vec(),
        }],
    };
    assert_eq!(
        ids(&group_failures(&today.icon_groups())),
        set(&["one icon group"]),
        "AC-1 control: resources with no RT_GROUP_ICON must fail the group count, and only it"
    );
    assert_eq!(
        ids(&group_failures(&[])),
        set(&["one icon group"]),
        "AC-1 control: no resource section at all must fail the group count too"
    );
}

#[test]
fn two_icon_groups_fail_ac1_on_the_group_count() {
    let blob = group_blob(&[(256, BPP, 1)]);
    assert_eq!(
        ids(&group_failures(&[&blob, &blob])),
        set(&["one icon group"]),
        "AC-1 control: two icon groups are not exactly one"
    );
}

#[test]
fn a_group_listing_only_the_256_fails_ac1_on_the_four_missing_sizes() {
    let blob = group_blob(&[(256, BPP, 1)]);
    assert_eq!(
        ids(&group_failures(&[&blob])),
        set(&[
            "image count",
            "size 16 listed",
            "size 24 listed",
            "size 32 listed",
            "size 48 listed",
        ]),
        "AC-1 control: a 256-only group must fail the count and each missing size, nothing else"
    );
}

#[test]
fn a_group_of_the_five_sizes_at_32_bpp_passes_ac1() {
    let blob = group_blob(&[
        (16, BPP, 1),
        (24, BPP, 2),
        (32, BPP, 3),
        (48, BPP, 4),
        (256, BPP, 5),
    ]);
    let failures = group_failures(&[&blob]);
    assert!(
        failures.is_empty(),
        "positive control: the group the story asks for fails AC-1:{}",
        report(&failures)
    );
    let entries = parse_group(&blob).expect("parse the control group");
    assert_eq!(
        entries.iter().map(GroupEntry::size).collect::<Vec<_>>(),
        SIZES.iter().map(|&s| (s, s)).collect::<Vec<_>>(),
        "a width/height byte of 0 must read as 256"
    );
}

#[test]
fn a_group_with_an_8_bpp_image_or_a_64_px_extra_fails_ac1() {
    let low = group_blob(&[
        (16, 8, 1),
        (24, BPP, 2),
        (32, BPP, 3),
        (48, BPP, 4),
        (256, BPP, 5),
    ]);
    assert_eq!(
        ids(&group_failures(&[&low])),
        set(&["32 bpp @16"]),
        "AC-1 control: an 8 bpp 16 px image must fail its bit depth, and only that"
    );
    let extra = group_blob(&[
        (16, BPP, 1),
        (24, BPP, 2),
        (32, BPP, 3),
        (48, BPP, 4),
        (64, BPP, 6),
        (256, BPP, 5),
    ]);
    assert_eq!(
        ids(&group_failures(&[&extra])),
        set(&["image count", "unexpected size (id 6)"]),
        "AC-1 control: a sixth, 64 px image is out of scope and must fail"
    );
}

#[test]
fn a_malformed_group_header_fails_ac1() {
    let mut blob = group_blob(&[(256, BPP, 1)]);
    blob[2] = 2; // type 2 is a cursor group
    assert_eq!(ids(&group_failures(&[&blob])), set(&["group header"]));
    let truncated = &group_blob(&[(16, BPP, 1), (24, BPP, 2)])[..20];
    assert_eq!(ids(&group_failures(&[truncated])), set(&["group header"]));
}

#[test]
fn five_pngs_of_their_declared_sizes_pass_ac2() {
    let (entries, icons) = five(passing);
    let mut failures = structure_failures(&entries, &icons);
    failures.extend(sample_failures(&entries, &icons));
    assert!(
        failures.is_empty(),
        "positive control: five PNGs drawn to AC-2 fail it:{}",
        report(&failures)
    );
}

#[test]
fn an_entry_whose_png_is_not_the_declared_size_fails_ac2() {
    // The 48 entry holds a 32 x 32 PNG.
    let (entries, icons) = five(|edge| passing(if edge == 48 { 32 } else { edge }));
    assert_eq!(
        ids(&structure_failures(&entries, &icons)),
        set(&["decoded size @48"]),
        "AC-2 control: a 32 px PNG declared as 48 must fail the declared-size check, and only it"
    );
}

#[test]
fn an_entry_that_is_a_dib_not_a_png_fails_ac2() {
    let (entries, mut icons) = five(passing);
    // A BITMAPINFOHEADER starts with its own size, 40.
    let mut dib = 40u32.to_le_bytes().to_vec();
    dib.extend([0u8; 36]);
    icons.insert(3, dib);
    assert_eq!(
        ids(&structure_failures(&entries, &icons)),
        set(&["png @32"]),
        "AC-2 control: a DIB entry must fail the PNG check"
    );
}

#[test]
fn a_group_pointing_at_a_missing_rt_icon_fails_ac2() {
    let (entries, mut icons) = five(passing);
    icons.remove(&1);
    assert_eq!(
        ids(&structure_failures(&entries, &icons)),
        set(&["RT_ICON present @16"]),
        "AC-2 control: a group entry with no RT_ICON behind it must fail"
    );
}

#[test]
fn a_solid_blue_entry_fails_ac2_on_pixel_0_0() {
    let (entries, icons) = five(passing);
    let mut icons = icons;
    icons.insert(3, png(&solid(32)));
    assert_eq!(
        ids(&sample_failures(&entries, &icons)),
        set(&["(0,0) alpha @32"]),
        "AC-2 control: a solid #0067C0 32 px entry must fail pixel (0,0), and only it"
    );
    let (entries, icons) = five(solid);
    assert_eq!(
        ids(&sample_failures(&entries, &icons)),
        SIZES.iter().map(|s| format!("(0,0) alpha @{s}")).collect(),
        "AC-2 control: solid entries at every size fail (0,0) at every size"
    );
}

#[test]
fn the_tile_sample_allows_2_per_channel_at_16_and_24_and_nothing_at_32_48_256() {
    // Each size in turn gets its tile sample nudged by `delta` on one channel.
    let nudged = |target: u32, channel: usize, delta: i16, alpha: u8| {
        let (entries, icons) = five(|edge| {
            let mut img = passing(edge);
            if edge == target {
                let &(_, (x, y), _) = TILE_SAMPLES
                    .iter()
                    .find(|(s, _, _)| *s == edge)
                    .expect("a sample size");
                let mut px = img.get_pixel(x, y).0;
                px[channel] = u8::try_from(i16::from(px[channel]) + delta).expect("in range");
                px[3] = alpha;
                img.put_pixel(x, y, image::Rgba(px));
            }
            img
        });
        ids(&sample_failures(&entries, &icons))
    };
    let none = BTreeSet::new();
    for size in [16, 24] {
        let at = TILE_SAMPLES.iter().find(|t| t.0 == size).expect("sample").1;
        let id = set(&[&format!("tile ({},{}) @{size}", at.0, at.1)]);
        assert_eq!(nudged(size, 0, 2, 255), none, "red +2 at {size} is inside");
        assert_eq!(
            nudged(size, 1, -2, 255),
            none,
            "green -2 at {size} is inside"
        );
        assert_eq!(nudged(size, 2, 2, 255), none, "blue +2 at {size} is inside");
        assert_eq!(nudged(size, 0, 3, 255), id, "red +3 at {size} is outside");
        assert_eq!(nudged(size, 1, 3, 255), id, "green +3 at {size} is outside");
        assert_eq!(nudged(size, 2, -3, 255), id, "blue -3 at {size} is outside");
        assert_eq!(
            nudged(size, 1, 0, 254),
            id,
            "alpha 254 at {size} is not opaque"
        );
    }
    for size in [32, 48, 256] {
        let at = TILE_SAMPLES.iter().find(|t| t.0 == size).expect("sample").1;
        let id = set(&[&format!("tile ({},{}) @{size}", at.0, at.1)]);
        assert_eq!(nudged(size, 0, 1, 255), id, "red +1 at {size} is outside");
        assert_eq!(
            nudged(size, 1, -1, 255),
            id,
            "green -1 at {size} is outside"
        );
        assert_eq!(nudged(size, 2, -1, 255), id, "blue -1 at {size} is outside");
        assert_eq!(
            nudged(size, 1, 0, 254),
            id,
            "alpha 254 at {size} is not opaque"
        );
    }
}

/// The window icon as a PNG entry, keyed as the fifth image of `five`.
fn with_256(window_png: Vec<u8>) -> (Vec<GroupEntry>, BTreeMap<u16, Vec<u8>>) {
    let (entries, mut icons) = five(passing);
    icons.insert(5, window_png);
    (entries, icons)
}

fn window_png(icon: &IconData) -> Vec<u8> {
    let img = RgbaImage::from_raw(icon.width, icon.height, icon.rgba.clone())
        .expect("the window icon is a whole RGBA raster");
    png(&img)
}

#[test]
fn a_256_entry_encoded_from_the_window_icon_passes_ac3() {
    let window = window_icon();
    let (entries, icons) = with_256(window_png(&window));
    let failures = identity_failures(&entries, &icons, &window);
    assert!(
        failures.is_empty(),
        "positive control: the window icon's own pixels fail AC-3:{}",
        report(&failures)
    );
}

#[test]
fn a_window_icon_with_pixel_128_128_altered_fails_ac3() {
    let window = window_icon();
    let (entries, icons) = with_256(window_png(&window));
    let mut altered = window.clone();
    let at = (128 * 256 + 128) * 4;
    altered.rgba[at] ^= 0x01;
    let failures = identity_failures(&entries, &icons, &altered);
    assert_eq!(
        ids(&failures),
        set(&["same pixels as the window icon"]),
        "AC-3 control: one altered pixel must fail the identity check"
    );
    assert!(
        failures[0].detail.contains("1 of 65536") && failures[0].detail.contains("(128,128)"),
        "AC-3 control: the failure names the one pixel that differs: {}",
        failures[0].detail
    );
}

#[test]
fn a_256_entry_encoded_from_premultiplied_pixels_fails_ac3() {
    // The renderer holds premultiplied colour; forgetting to demultiply
    // changes every anti-aliased edge pixel and nothing at the samples.
    let window = window_icon();
    let mut premultiplied = window.clone();
    for px in premultiplied.rgba.chunks_mut(4) {
        let a = u16::from(px[3]);
        for c in &mut px[..3] {
            *c = u8::try_from((u16::from(*c) * a + 127) / 255).expect("fits a byte");
        }
    }
    assert_ne!(
        premultiplied.rgba, window.rgba,
        "the window icon has partly transparent coloured pixels to tell the two apart"
    );
    let (entries, icons) = with_256(window_png(&premultiplied));
    assert_eq!(
        ids(&identity_failures(&entries, &icons, &window)),
        set(&["same pixels as the window icon"]),
        "AC-3 control: a premultiplied 256 entry must fail the identity check"
    );
}

#[test]
fn a_group_with_no_256_entry_fails_ac3() {
    let window = window_icon();
    let entries = parse_group(&group_blob(&[(16, BPP, 1)])).expect("a control group");
    assert_eq!(
        ids(&identity_failures(&entries, &BTreeMap::new(), &window)),
        set(&["256 entry"]),
    );
}
