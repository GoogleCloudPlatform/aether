//! Mathematical operations runtime support

use std::ffi::c_double;

/// Power function
#[no_mangle]
pub extern "C" fn aether_pow(base: c_double, exponent: c_double) -> c_double {
    base.powf(exponent)
}

/// Square root
#[no_mangle]
pub extern "C" fn aether_sqrt(x: c_double) -> c_double {
    x.sqrt()
}

/// Sine
#[no_mangle]
pub extern "C" fn aether_sin(x: c_double) -> c_double {
    x.sin()
}

/// Cosine
#[no_mangle]
pub extern "C" fn aether_cos(x: c_double) -> c_double {
    x.cos()
}

/// Tangent
#[no_mangle]
pub extern "C" fn aether_tan(x: c_double) -> c_double {
    x.tan()
}

/// Natural logarithm
#[no_mangle]
pub extern "C" fn aether_log(x: c_double) -> c_double {
    x.ln()
}

/// Exponential
#[no_mangle]
pub extern "C" fn aether_exp(x: c_double) -> c_double {
    x.exp()
}

/// Arc sine
#[no_mangle]
pub extern "C" fn aether_asin(x: c_double) -> c_double {
    x.asin()
}

/// Arc cosine
#[no_mangle]
pub extern "C" fn aether_acos(x: c_double) -> c_double {
    x.acos()
}

/// Arc tangent
#[no_mangle]
pub extern "C" fn aether_atan(x: c_double) -> c_double {
    x.atan()
}

/// Arc tangent of y/x
#[no_mangle]
pub extern "C" fn aether_atan2(y: c_double, x: c_double) -> c_double {
    y.atan2(x)
}

/// Hyperbolic sine
#[no_mangle]
pub extern "C" fn aether_sinh(x: c_double) -> c_double {
    x.sinh()
}

/// Hyperbolic cosine
#[no_mangle]
pub extern "C" fn aether_cosh(x: c_double) -> c_double {
    x.cosh()
}

/// Hyperbolic tangent
#[no_mangle]
pub extern "C" fn aether_tanh(x: c_double) -> c_double {
    x.tanh()
}

/// Floor
#[no_mangle]
pub extern "C" fn aether_floor(x: c_double) -> c_double {
    x.floor()
}

/// Ceiling
#[no_mangle]
pub extern "C" fn aether_ceil(x: c_double) -> c_double {
    x.ceil()
}

/// Round
#[no_mangle]
pub extern "C" fn aether_round(x: c_double) -> c_double {
    x.round()
}

/// Absolute value for floating point
#[no_mangle]
pub extern "C" fn aether_fabs(x: c_double) -> c_double {
    x.abs()
}

/// Modulo for floating point
#[no_mangle]
pub extern "C" fn aether_fmod(x: c_double, y: c_double) -> c_double {
    x % y
}