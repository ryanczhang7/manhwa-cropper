//! MC-001: proves the test runner discovers `crates/engine/tests/` and that
//! the engine links against `cropper-core`. Not product behaviour.

use cropper_core::Dimensions;
use cropper_engine::{MAX_DIMENSIONS, within_limits};

#[test]
fn limits_admit_4k_and_refuse_anything_wider_or_taller_than_8k() {
    let uhd = Dimensions {
        width: 3840,
        height: 2160,
    };
    assert!(within_limits(uhd));
    assert!(within_limits(MAX_DIMENSIONS));

    let too_wide = Dimensions {
        width: MAX_DIMENSIONS.width + 1,
        height: 1,
    };
    assert!(!within_limits(too_wide));

    let too_tall = Dimensions {
        width: 1,
        height: MAX_DIMENSIONS.height + 1,
    };
    assert!(!within_limits(too_tall));
}
