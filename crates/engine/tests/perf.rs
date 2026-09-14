//! MC-017, AC-1 to AC-3: one hundred screenshots crop in under ten seconds.
//!
//! One run of [`batch::run`](cropper_engine::batch::run) over 100 synthetic
//! RGB8 PNGs - 40 at 1920x1080, 30 at 2560x1440, 30 at 3840x2160 - and three
//! assertions about it: the wall time of `run` alone (AC-1), the dimensions of
//! every output (AC-2), and that nothing failed or was flagged (AC-3). All
//! three describe *the same run*, which is why they are one test: the batch
//! writes 1.3 GB of fixtures and reads them back, and running it three times
//! would buy nothing but three names.
//!
//! The test is `#[ignore]`d, so `cargo test --workspace` (the `unit` gate,
//! debug) never runs it. The `integration` gate does, in release:
//!
//! ```text
//! cargo test --workspace --release -- --ignored
//! ```
//!
//! and to run it alone, with the measured time on stdout:
//!
//! ```text
//! cargo test -p cropper-engine --release --test perf -- --ignored --nocapture
//! ```
//!
//! **Release is not optional.** Image decode in an unoptimised build is an
//! order of magnitude slower than in a release one, so a debug measurement
//! against a 10 s budget measures the profile and not the engine.
//!
//! # What is settled, what is mechanical, what was measured
//!
//! * **Settled**, read out of the criteria and never re-derived: the 10.0 s
//!   budget, the 100 images, and the 40/30/30 mix at 1920x1080 / 2560x1440 /
//!   3840x2160. [`BUDGET`], [`TOTAL`] and [`RECIPES`] hold them, and
//!   [`the_settled_mix_is_what_is_generated`] fails if the table ever stops
//!   adding up to the criterion.
//! * **Mechanical**: AC-2 and AC-3. Each recipe carries a *known* art rect, so
//!   the expected output size is computed from the recipe and
//!   `Tuning::margin_px` - never read off what the engine produced - and the
//!   size actually written is read out of each PNG's own IHDR rather than from
//!   a decoder's opinion of it. `margin_px` is read from the `Tuning` the run
//!   was given, so a change to the tuning moves the expectation instead of
//!   breaking it.
//! * **Oracle-free**: the timing. There is no settled answer for how long this
//!   machine takes, so what matters is that the *metric* is honest: the timer
//!   goes around `run` and nothing else. Writing the fixtures (measured at
//!   ~1.0 s, half the run itself) and reading the outputs back for AC-2 are
//!   both this test's own overhead, and both sit outside the timed region.
//!
//! # Why AC-2 is the control on AC-1's clock
//!
//! A clock on its own cannot tell work from the absence of work: a stub that
//! copied these inputs instead of cropping them was measured at **0.270 s**
//! against the 10.0 s budget, seven times faster than the real run. AC-2 is
//! what makes the measured time mean something, because it is checked on the
//! same run: 100 outputs, each exactly its recipe's art rect plus
//! `2 * margin_px` on each axis. The copying stub lands on the *input*
//! dimensions instead - 21x24 px too large at 1080p, 75x84 px at 4K - and
//! AC-3 catches the other cheap way out, because flagging every file copies it
//! through untouched and is faster still. The measurements are in the story's
//! `## Handoff: RED -> GREEN`.
//!
//! # Why the scene is built here and not in `common`
//!
//! [`common::screenshot`] is one fixed 147x120 scene by design, and every
//! other engine suite pins numbers measured on it. This story needs the same
//! *structure* - four uniform borders, a chrome band below the top one,
//! textured art inside - at three resolutions it chooses, so [`scene`] builds
//! it from a [`Recipe`] while reusing `common`'s colours, its seeded
//! [`Rng`](common::Rng) and its writers. Parameterising `screenshot` itself
//! would have changed the fixture five other suites measure.

mod common;

use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use cropper_core::{Luma, Rect, Tuning};
use cropper_engine::batch::{Progress, RunSummary, run};
use cropper_engine::{FileResult, Outcome};

// --- The settled numbers ----------------------------------------------------

/// The budget AC-1 gives the whole batch, from the product brief: below 10.0 s
/// for `run` alone. Settled - if this is wrong it goes back to the product
/// owner, never down.
const BUDGET: Duration = Duration::from_secs(10);

/// How many screenshots the batch is given. Settled.
const TOTAL: usize = 100;

// --- The recipes ------------------------------------------------------------

/// One resolution class: an image size, the four border depths, the chrome
/// band, how many of them the batch gets, and the seed their art is drawn
/// from.
///
/// The borders are what the detector trims and the band is what it peels, so
/// the art rect - and therefore the expected output size - is known before a
/// single pixel is written. That is what AC-2 needs and what an "observe the
/// output and assert it again" test would not give.
#[derive(Debug, Clone, Copy)]
struct Recipe {
    /// The name a failure message calls this class.
    name: &'static str,
    /// Image width, in pixels.
    width: u32,
    /// Image height, in pixels.
    height: u32,
    /// Depth of the uniform top border.
    top: u32,
    /// Height of the chrome band, immediately below the top border.
    band: u32,
    /// Depth of the uniform bottom border.
    bottom: u32,
    /// Depth of the uniform left border.
    left: u32,
    /// Depth of the uniform right border.
    right: u32,
    /// How many images of this class the batch is given.
    count: usize,
    /// The base seed for this class's art texture; image `i` uses
    /// `seed * 1000 + i`.
    seed: u32,
}

impl Recipe {
    /// The art rect in image coordinates: inside the borders, below the band.
    fn art(self) -> Rect {
        Rect {
            x: self.left,
            y: self.top + self.band,
            w: self.width - self.left - self.right,
            h: self.height - self.top - self.band - self.bottom,
        }
    }

    /// What a correct crop of this recipe must measure: the art rect expanded
    /// by `margin` on every side. Not clamped, because
    /// [`margins_fit_inside_every_border`] proves every border is deeper than
    /// the margin, so no side of the expansion can reach an edge.
    fn expected_output(self, margin: u32) -> (u32, u32) {
        let art = self.art();
        (art.w + 2 * margin, art.h + 2 * margin)
    }

    /// The seed image `index` of this class draws its art from.
    fn seed_for(self, index: usize) -> u32 {
        self.seed * 1000 + u32::try_from(index).expect("fewer than 2^32 images per recipe")
    }
}

/// AC-1's mix, whole: 40 at 1920x1080, 30 at 2560x1440, 30 at 3840x2160.
///
/// The border depths and the band height are this test's own choice - the
/// criterion says "uniform borders and a chrome band" and fixes nothing else -
/// and they scale with the image so that a 4K screenshot does not have a
/// 10 px frame. The base seeds are 1, 7 and 42.
const RECIPES: [Recipe; 3] = [
    Recipe {
        name: "1920x1080",
        width: 1920,
        height: 1080,
        top: 10,
        band: 12,
        bottom: 8,
        left: 15,
        right: 12,
        count: 40,
        seed: 1,
    },
    Recipe {
        name: "2560x1440",
        width: 2560,
        height: 1440,
        top: 20,
        band: 24,
        bottom: 16,
        left: 30,
        right: 24,
        count: 30,
        seed: 7,
    },
    Recipe {
        name: "3840x2160",
        width: 3840,
        height: 2160,
        top: 30,
        band: 36,
        bottom: 24,
        left: 45,
        right: 36,
        count: 30,
        seed: 42,
    },
];

/// How many images of each class one round of the plan lays down: 4, 3, 3,
/// which is the mix in miniature.
const PER_ROUND: [usize; 3] = [4, 3, 3];

// --- Building the batch -----------------------------------------------------

/// The order the batch is given, as `(recipe, image within that recipe)`
/// pairs: the three classes interleaved 4-3-3 per round, so the folder reads
/// like a mixed capture session rather than three blocks, and no worker's
/// share of the pool is all 4K.
fn plan() -> Vec<(usize, usize)> {
    let mut next = [0usize; RECIPES.len()];
    let mut plan = Vec::with_capacity(TOTAL);
    while next
        .iter()
        .zip(RECIPES)
        .any(|(done, recipe)| *done < recipe.count)
    {
        for (index, &take) in PER_ROUND.iter().enumerate() {
            for _ in 0..take {
                if next[index] < RECIPES[index].count {
                    plan.push((index, next[index]));
                    next[index] += 1;
                }
            }
        }
    }
    plan
}

/// The screenshot scene for `recipe`, drawn with `seed`: four uniform borders,
/// a chrome band spanning the width between the side borders, and a jittered
/// checkerboard inside.
///
/// The same construction as [`common::screenshot`], at a size the recipe
/// chooses. The top and bottom borders span the full width and the side
/// borders only the rows between them, so the first uniform pass can reach
/// only the top and bottom and the sides go on the pass after - which is what
/// makes this a screenshot rather than a frame.
fn scene(recipe: Recipe, seed: u32) -> Luma {
    let w = recipe.width as usize;
    let h = recipe.height as usize;
    let mut data = vec![0u8; w * h];
    let art = recipe.art();

    // Art: a jittered checkerboard. Nothing in it is flat enough at any depth
    // to be mistaken for chrome, which is what keeps the art rect the answer.
    let mut rng = common::Rng::new(seed);
    for y in art.y..art.y + art.h {
        let row = y as usize * w;
        for x in art.x..art.x + art.w {
            let base = if (x + y).is_multiple_of(2) {
                common::ART_LO
            } else {
                common::ART_HI
            };
            data[row + x as usize] = base + rng.upto(common::ART_JITTER);
        }
    }

    // Chrome band: a flat background with one pixel in every `TEXT_PERIOD`
    // moved off it, alternately up and down so the band's median stays on the
    // background and its flat fraction stays above `chrome_flat_fraction`.
    for row in 0..recipe.band {
        let y = recipe.top + row;
        let mut moved = 0u32;
        for i in 0..art.w {
            let x = recipe.left + i;
            let v = if (i + 3 * row).is_multiple_of(common::TEXT_PERIOD) {
                let up = moved.is_multiple_of(2);
                moved += 1;
                if up {
                    common::BAND_BG + common::BAND_DEVIATION
                } else {
                    common::BAND_BG - common::BAND_DEVIATION
                }
            } else {
                common::BAND_BG
            };
            data[y as usize * w + x as usize] = v;
        }
    }

    // Borders. Top and bottom span the full width; the sides fill only the
    // rows between them.
    for y in 0..recipe.top {
        data[y as usize * w..(y as usize + 1) * w].fill(common::TOP_COLOUR);
    }
    for y in recipe.height - recipe.bottom..recipe.height {
        data[y as usize * w..(y as usize + 1) * w].fill(common::BOTTOM_COLOUR);
    }
    for y in recipe.top..recipe.height - recipe.bottom {
        let row = y as usize * w;
        data[row..row + recipe.left as usize].fill(common::LEFT_COLOUR);
        data[row + w - recipe.right as usize..row + w].fill(common::RIGHT_COLOUR);
    }

    Luma {
        width: recipe.width,
        height: recipe.height,
        data,
    }
}

/// Write every image the plan asks for into `dir` and answer with their paths,
/// in plan order.
///
/// Spread over the machine's threads because this is the expensive half of the
/// test - hundreds of megapixels of PNG - and none of it is inside the timed
/// region. `std::thread` rather than rayon: the test crate has no rayon
/// dependency, and borrowing it from the engine's would be borrowing the very
/// thing AC-1 measures.
fn generate(dir: &Path, plan: &[(usize, usize)]) -> Vec<PathBuf> {
    let paths: Vec<PathBuf> = (0..plan.len())
        .map(|i| dir.join(format!("shot_{i:03}.png")))
        .collect();
    let jobs: Vec<(&PathBuf, (usize, usize))> = paths.iter().zip(plan.iter().copied()).collect();
    let threads = std::thread::available_parallelism().map_or(4, NonZeroUsize::get);
    let chunk = jobs.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        for slice in jobs.chunks(chunk) {
            scope.spawn(move || {
                for &(path, (recipe, index)) in slice {
                    let recipe = RECIPES[recipe];
                    common::write_rgb8(&scene(recipe, recipe.seed_for(index)), path);
                }
            });
        }
    });
    paths
}

// --- Reading the run back ---------------------------------------------------

/// The output path of a result, or `None` when the file produced no output.
fn output_of(result: &FileResult) -> Option<&Path> {
    match &result.outcome {
        Outcome::Cropped { output, .. } | Outcome::Flagged { output, .. } => Some(output.as_path()),
        Outcome::Failed { .. } => None,
    }
}

/// One line per result that was not cropped, naming the file and what happened
/// to it instead - a failure message that reads like a bug report rather than
/// a count.
fn not_cropped(summary: &RunSummary) -> Vec<String> {
    summary
        .results
        .iter()
        .filter_map(|result| {
            let name = result
                .input
                .file_name()
                .unwrap_or(result.input.as_os_str())
                .to_string_lossy();
            match &result.outcome {
                Outcome::Cropped { .. } => None,
                Outcome::Flagged { reason, .. } => Some(format!("{name}: flagged {reason:?}")),
                Outcome::Failed { error } => Some(format!("{name}: failed: {error}")),
            }
        })
        .collect()
}

/// How many files are in `dir`.
fn count_entries(dir: &Path) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
        .count()
}

/// How many megabytes of files are in `dir` - diagnostic only, and never
/// inside the timed region. A time without a volume is a number nobody can
/// compare against anything.
fn megabytes_in(dir: &Path) -> f64 {
    let bytes: u64 = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("reading {}: {err}", dir.display()))
        .filter_map(|entry| entry.ok()?.metadata().ok())
        .map(|meta| meta.len())
        .sum();
    bytes as f64 / (1024.0 * 1024.0)
}

// --- The settled mix, asserted against the table ----------------------------

/// Cheap, not ignored, and not a measurement: if the table above ever stops
/// being AC-1's mix, every number this file reports is about a different
/// batch. Debug is fine for it - it writes nothing and decodes nothing.
#[test]
fn the_settled_mix_is_what_is_generated() {
    let counts: Vec<usize> = RECIPES.iter().map(|recipe| recipe.count).collect();
    assert_eq!(
        counts,
        vec![40, 30, 30],
        "AC-1's mix is 40 / 30 / 30; the recipe table says {counts:?}"
    );
    let sizes: Vec<(u32, u32)> = RECIPES
        .iter()
        .map(|recipe| (recipe.width, recipe.height))
        .collect();
    assert_eq!(
        sizes,
        vec![(1920, 1080), (2560, 1440), (3840, 2160)],
        "AC-1's resolutions are 1920x1080, 2560x1440 and 3840x2160; the recipe table says {sizes:?}"
    );
    let plan = plan();
    assert_eq!(
        plan.len(),
        TOTAL,
        "AC-1 gives the batch {TOTAL} screenshots; the plan lays out {}",
        plan.len()
    );
    for (index, recipe) in RECIPES.iter().enumerate() {
        let planned = plan.iter().filter(|(which, _)| *which == index).count();
        assert_eq!(
            planned, recipe.count,
            "the plan holds {planned} of the {} {} screenshots",
            recipe.count, recipe.name
        );
    }
}

/// Also cheap, also not a measurement: AC-2 expects the art rect plus
/// `2 * margin_px` on each axis, with no clamping anywhere. That is only true
/// while every border is at least as deep as the margin, and the margin comes
/// from the tuning rather than from this file.
#[test]
fn margins_fit_inside_every_border() {
    let margin = Tuning::default().margin_px;
    for recipe in RECIPES {
        let shallowest = recipe
            .top
            .min(recipe.bottom)
            .min(recipe.left)
            .min(recipe.right);
        assert!(
            shallowest >= margin,
            "{}'s shallowest border is {shallowest} px and the margin is {margin} px, so AC-2's expected size would be clamped on that side",
            recipe.name
        );
    }
}

// --- AC-1, AC-2, AC-3: one run ----------------------------------------------

#[test]
#[ignore = "1.3 GB of fixtures and a release-only timing measurement; run by the integration gate"]
fn one_hundred_mixed_screenshots_are_cropped_correctly_within_the_ten_second_budget() {
    let tuning = Tuning::default();
    let margin = tuning.margin_px;

    let tmp = tempfile::tempdir().expect("a temp dir");
    let src = tmp.path().join("in");
    fs::create_dir(&src).expect("an input dir");
    let out = tmp.path().join("out");

    let plan = plan();
    let fixtures_started = Instant::now();
    let inputs = generate(&src, &plan);
    let fixtures = fixtures_started.elapsed();
    assert_eq!(inputs.len(), TOTAL, "the batch is given {TOTAL} files");

    // The timed region, and nothing else in it: no fixture generation above,
    // no decoding below.
    let started = Instant::now();
    let summary = run(&inputs, &out, &tuning, &|_: Progress| {});
    let elapsed = started.elapsed();

    // Printed rather than only asserted, because AC-1 asks for the measured
    // time to be recorded whether or not it is inside the budget. Visible with
    // `-- --nocapture`, and in the failure message below either way.
    println!(
        "MC-017: batch::run took {:.3} s of the {:.1} s budget ({TOTAL} files, {:.0} MB in, {:.0} MB out; fixtures took a further {:.1} s, outside the timed region)",
        elapsed.as_secs_f64(),
        BUDGET.as_secs_f64(),
        megabytes_in(&src),
        megabytes_in(&out),
        fixtures.as_secs_f64()
    );

    // AC-3 first: a run that flagged or failed its way to a fast time has not
    // done the work the clock is supposed to be measuring.
    let unhappy = not_cropped(&summary);
    assert!(
        unhappy.is_empty(),
        "AC-3: {} of {TOTAL} screenshots were not cropped (failed {}, flagged {}):\n  {}",
        unhappy.len(),
        summary.failed(),
        summary.flagged(),
        unhappy.join("\n  ")
    );

    // AC-2: one output per input, each exactly its recipe's art rect plus the
    // margin on all four sides. Read out of each PNG's IHDR - the file's own
    // answer - and cross-checked against a real decode once per recipe below.
    assert_eq!(
        count_entries(&out),
        TOTAL,
        "AC-2: the output folder holds {} files, not {TOTAL}",
        count_entries(&out)
    );
    let mut wrong = Vec::new();
    for (position, &(which, index)) in plan.iter().enumerate() {
        let recipe = RECIPES[which];
        let expected = recipe.expected_output(margin);
        let output = output_of(&summary.results[position])
            .unwrap_or_else(|| panic!("{} produced no output", inputs[position].display()));
        let header = common::ihdr(output);
        if (header.width, header.height) != expected {
            wrong.push(format!(
                "{} ({}, image {index}): {}x{} written where the art rect {}x{} plus 2 * margin_px ({margin}) is {}x{}",
                output.file_name().unwrap_or(output.as_os_str()).to_string_lossy(),
                recipe.name,
                header.width,
                header.height,
                recipe.art().w,
                recipe.art().h,
                expected.0,
                expected.1
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "AC-2: {} of {TOTAL} outputs are not their recipe's art rect plus the margin:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );

    // The IHDR above is the container's own claim about the size. Once per
    // recipe, make a decoder agree with it: a file whose header and pixels
    // disagree would satisfy the loop above and be unopenable.
    for (which, recipe) in RECIPES.iter().enumerate() {
        let position = plan
            .iter()
            .position(|(r, _)| *r == which)
            .expect("every recipe appears in the plan");
        let output = output_of(&summary.results[position]).expect("an output to decode");
        let decoded = common::decode(output);
        assert_eq!(
            (decoded.width(), decoded.height()),
            recipe.expected_output(margin),
            "AC-2: {} decodes to {}x{} where its IHDR and its recipe say {:?}",
            output.display(),
            decoded.width(),
            decoded.height(),
            recipe.expected_output(margin)
        );
    }

    // AC-1 last, on a run that has just been shown to have done the work.
    assert!(
        elapsed < BUDGET,
        "AC-1: batch::run took {:.3} s over {TOTAL} screenshots (40 at 1920x1080, 30 at 2560x1440, 30 at 3840x2160), past the {:.1} s budget. Fixture generation took {:.1} s and is outside the timed region.",
        elapsed.as_secs_f64(),
        BUDGET.as_secs_f64(),
        fixtures.as_secs_f64()
    );
}
