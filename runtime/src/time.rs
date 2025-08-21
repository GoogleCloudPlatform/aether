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

//! Time and date operations runtime support

use std::ffi::{c_char, c_int};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, TimeZone, Datelike, Timelike, FixedOffset};
use std::ptr;

/// DateTime structure matching AetherScript definition
#[repr(C)]
pub struct AetherDateTime {
    year: c_int,
    month: c_int,
    day: c_int,
    hour: c_int,
    minute: c_int,
    second: c_int,
    timezone_offset: c_int, // Minutes from UTC
}

/// Get current Unix timestamp
#[no_mangle]
pub extern "C" fn aether_time_now() -> c_int {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as c_int,
        Err(_) => 0,
    }
}

/// Convert timestamp to datetime
#[no_mangle]
pub unsafe extern "C" fn aether_timestamp_to_datetime(
    timestamp: c_int,
    timezone_offset: c_int,
    datetime: *mut AetherDateTime
) {
    if datetime.is_null() {
        return;
    }
    
    let timestamp = timestamp as i64;
    let offset = FixedOffset::east_opt(timezone_offset * 60).unwrap_or(FixedOffset::east_opt(0).unwrap());
    
    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        let dt_with_offset = dt.with_timezone(&offset);
        
        (*datetime).year = dt_with_offset.year();
        (*datetime).month = dt_with_offset.month() as c_int;
        (*datetime).day = dt_with_offset.day() as c_int;
        (*datetime).hour = dt_with_offset.hour() as c_int;
        (*datetime).minute = dt_with_offset.minute() as c_int;
        (*datetime).second = dt_with_offset.second() as c_int;
        (*datetime).timezone_offset = timezone_offset;
    }
}

/// Convert datetime to timestamp
#[no_mangle]
pub unsafe extern "C" fn aether_datetime_to_timestamp(datetime: *const AetherDateTime) -> c_int {
    if datetime.is_null() {
        return 0;
    }
    
    let offset = FixedOffset::east_opt((*datetime).timezone_offset * 60).unwrap_or(FixedOffset::east_opt(0).unwrap());
    
    match offset.with_ymd_and_hms(
        (*datetime).year,
        (*datetime).month as u32,
        (*datetime).day as u32,
        (*datetime).hour as u32,
        (*datetime).minute as u32,
        (*datetime).second as u32
    ) {
        chrono::LocalResult::Single(dt) => dt.timestamp() as c_int,
        _ => 0,
    }
}

/// Format datetime as ISO 8601 string
#[no_mangle]
pub unsafe extern "C" fn aether_format_datetime_iso8601(
    datetime: *const AetherDateTime,
    buffer: *mut c_char
) {
    if datetime.is_null() || buffer.is_null() {
        return;
    }
    
    let offset = FixedOffset::east_opt((*datetime).timezone_offset * 60).unwrap_or(FixedOffset::east_opt(0).unwrap());
    
    match offset.with_ymd_and_hms(
        (*datetime).year,
        (*datetime).month as u32,
        (*datetime).day as u32,
        (*datetime).hour as u32,
        (*datetime).minute as u32,
        (*datetime).second as u32
    ) {
        chrono::LocalResult::Single(dt) => {
            let iso_string = dt.to_rfc3339();
            let bytes = iso_string.as_bytes();
            
            ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, buffer, bytes.len());
            *buffer.offset(bytes.len() as isize) = 0; // Null terminate
        },
        _ => {
            *buffer = 0; // Empty string on error
        }
    }
}

/// Parse ISO 8601 string to datetime
#[no_mangle]
pub unsafe extern "C" fn aether_parse_datetime_iso8601(
    str: *const c_char,
    datetime: *mut AetherDateTime
) -> c_int {
    if str.is_null() || datetime.is_null() {
        return 0;
    }
    
    let s = match std::ffi::CStr::from_ptr(str).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    match DateTime::parse_from_rfc3339(s) {
        Ok(dt) => {
            (*datetime).year = dt.year();
            (*datetime).month = dt.month() as c_int;
            (*datetime).day = dt.day() as c_int;
            (*datetime).hour = dt.hour() as c_int;
            (*datetime).minute = dt.minute() as c_int;
            (*datetime).second = dt.second() as c_int;
            (*datetime).timezone_offset = dt.offset().local_minus_utc() / 60;
            1 // Success
        },
        Err(_) => 0, // Failure
    }
}

/// Sleep for specified milliseconds
#[no_mangle]
pub extern "C" fn aether_sleep_ms(milliseconds: c_int) {
    if milliseconds > 0 {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds as u64));
    }
}

/// Get high-resolution timer value in nanoseconds
#[no_mangle]
pub extern "C" fn aether_hrtime() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as i64,
        Err(_) => 0,
    }
}