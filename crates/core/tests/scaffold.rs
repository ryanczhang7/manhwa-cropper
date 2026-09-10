//! MC-001: proves the test runner discovers `crates/core/tests/` and that the
//! coverage gate measures `crates/core/src/lib.rs`. Not a slice of product
//! behaviour; those tests start at MC-003.

use cropper_core::Dimensions;

#[test]
fn area_is_width_times_height_without_overflow() {
    let hd = Dimensions {
        width: 1920,
        height: 1080,
    };
    assert_eq!(hd.area(), 2_073_600);

    // u32::MAX squared does not fit in u32; it must fit in the u64 result.
    let huge = Dimensions {
        width: u32::MAX,
        height: u32::MAX,
    };
    assert_eq!(huge.area(), u64::from(u32::MAX) * u64::from(u32::MAX));
}
