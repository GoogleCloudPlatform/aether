// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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