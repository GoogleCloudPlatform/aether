//! GGUF file loading and tensor access
//! Provides FFI functions for loading and reading GGUF model files

use lazy_static::lazy_static;
use libc::{c_char, c_int};
use memmap2::Mmap;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::File;
use std::ptr;
use std::sync::Mutex;

// GGUF constants
const GGUF_MAGIC: u32 = 0x46554747; // "GGUF"
const GGML_TYPE_F32: u32 = 0;

// Metadata value types
const GGUF_TYPE_UINT8: u32 = 0;
const GGUF_TYPE_INT8: u32 = 1;
const GGUF_TYPE_UINT16: u32 = 2;
const GGUF_TYPE_INT16: u32 = 3;
const GGUF_TYPE_UINT32: u32 = 4;
const GGUF_TYPE_INT32: u32 = 5;
const GGUF_TYPE_FLOAT32: u32 = 6;
const GGUF_TYPE_BOOL: u32 = 7;
const GGUF_TYPE_STRING: u32 = 8;
const GGUF_TYPE_ARRAY: u32 = 9;
const GGUF_TYPE_UINT64: u32 = 10;
const GGUF_TYPE_INT64: u32 = 11;
const GGUF_TYPE_FLOAT64: u32 = 12;

#[derive(Debug, Clone)]
struct TensorInfo {
    name: String,
    dims: Vec<u64>,
    dtype: u32,
    offset: u64, // Offset from start of tensor data section
}

struct GgufModel {
    _file: File,
    mmap: Mmap,
    version: u32,
    tensor_count: u64,
    metadata_kv_count: u64,
    tensors: HashMap<String, TensorInfo>,
    tensor_data_offset: usize, // Where tensor data starts in the file
}

lazy_static! {
    static ref LOADED_MODELS: Mutex<HashMap<i64, GgufModel>> = Mutex::new(HashMap::new());
    static ref NEXT_MODEL_ID: Mutex<i64> = Mutex::new(1);
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

fn read_f32(data: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

fn read_string(data: &[u8], offset: usize) -> (String, usize) {
    let len = read_u64(data, offset) as usize;
    let s = String::from_utf8_lossy(&data[offset + 8..offset + 8 + len]).to_string();
    (s, offset + 8 + len)
}

fn skip_metadata_value(data: &[u8], offset: usize, value_type: u32) -> usize {
    match value_type {
        GGUF_TYPE_UINT8 | GGUF_TYPE_INT8 | GGUF_TYPE_BOOL => offset + 1,
        GGUF_TYPE_UINT16 | GGUF_TYPE_INT16 => offset + 2,
        GGUF_TYPE_UINT32 | GGUF_TYPE_INT32 | GGUF_TYPE_FLOAT32 => offset + 4,
        GGUF_TYPE_UINT64 | GGUF_TYPE_INT64 | GGUF_TYPE_FLOAT64 => offset + 8,
        GGUF_TYPE_STRING => {
            let len = read_u64(data, offset) as usize;
            offset + 8 + len
        }
        GGUF_TYPE_ARRAY => {
            let elem_type = read_u32(data, offset);
            let count = read_u64(data, offset + 4) as usize;
            let mut pos = offset + 12;
            for _ in 0..count {
                pos = skip_metadata_value(data, pos, elem_type);
            }
            pos
        }
        _ => offset, // Unknown type
    }
}

fn parse_gguf(mmap: &Mmap) -> Result<(u32, u64, u64, HashMap<String, TensorInfo>, usize), String> {
    let data = &mmap[..];

    // Check magic
    let magic = read_u32(data, 0);
    if magic != GGUF_MAGIC {
        return Err(format!("Invalid GGUF magic: {:08x}", magic));
    }

    let version = read_u32(data, 4);
    let tensor_count = read_u64(data, 8);
    let metadata_kv_count = read_u64(data, 16);

    let mut offset = 24; // After header

    // Skip metadata
    for _ in 0..metadata_kv_count {
        // Read key
        let (_key, new_offset) = read_string(data, offset);
        offset = new_offset;

        // Read value type
        let value_type = read_u32(data, offset);
        offset += 4;

        // Skip value
        offset = skip_metadata_value(data, offset, value_type);
    }

    // Parse tensor infos
    let mut tensors = HashMap::new();
    for _ in 0..tensor_count {
        // Read tensor name
        let (name, new_offset) = read_string(data, offset);
        offset = new_offset;

        // Read n_dims
        let n_dims = read_u32(data, offset) as usize;
        offset += 4;

        // Read dimensions
        let mut dims = Vec::with_capacity(n_dims);
        for _ in 0..n_dims {
            dims.push(read_u64(data, offset));
            offset += 8;
        }

        // Read dtype
        let dtype = read_u32(data, offset);
        offset += 4;

        // Read tensor data offset
        let tensor_offset = read_u64(data, offset);
        offset += 8;

        tensors.insert(
            name.clone(),
            TensorInfo {
                name,
                dims,
                dtype,
                offset: tensor_offset,
            },
        );
    }

    // Align to 32 bytes for tensor data
    let aligned_offset = (offset + 31) & !31;

    Ok((version, tensor_count, metadata_kv_count, tensors, aligned_offset))
}

/// Load a GGUF model file
/// Returns model handle (>0) on success, <=0 on error
#[no_mangle]
pub unsafe extern "C" fn gguf_load(path: *const c_char) -> i64 {
    if path.is_null() {
        return -1;
    }

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    let file = match File::open(path_str) {
        Ok(f) => f,
        Err(_) => return -3,
    };

    let mmap = match Mmap::map(&file) {
        Ok(m) => m,
        Err(_) => return -4,
    };

    let (version, tensor_count, metadata_kv_count, tensors, tensor_data_offset) =
        match parse_gguf(&mmap) {
            Ok(result) => result,
            Err(_) => return -5,
        };

    let model = GgufModel {
        _file: file,
        mmap,
        version,
        tensor_count,
        metadata_kv_count,
        tensors,
        tensor_data_offset,
    };

    let mut models = LOADED_MODELS.lock().unwrap();
    let mut next_id = NEXT_MODEL_ID.lock().unwrap();
    let model_id = *next_id;
    *next_id += 1;
    models.insert(model_id, model);

    model_id
}

/// Unload a GGUF model
#[no_mangle]
pub extern "C" fn gguf_unload(model_id: i64) -> c_int {
    let mut models = LOADED_MODELS.lock().unwrap();
    if models.remove(&model_id).is_some() {
        0
    } else {
        -1
    }
}

/// Get the number of tensors in the model
#[no_mangle]
pub extern "C" fn gguf_tensor_count(model_id: i64) -> i64 {
    let models = LOADED_MODELS.lock().unwrap();
    match models.get(&model_id) {
        Some(model) => model.tensor_count as i64,
        None => -1,
    }
}

/// Get the number of elements in a tensor by name
/// Returns -1 if tensor not found
#[no_mangle]
pub unsafe extern "C" fn gguf_tensor_numel(model_id: i64, name: *const c_char) -> i64 {
    if name.is_null() {
        return -1;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let models = LOADED_MODELS.lock().unwrap();
    match models.get(&model_id) {
        Some(model) => match model.tensors.get(name_str) {
            Some(tensor) => tensor.dims.iter().product::<u64>() as i64,
            None => -1,
        },
        None => -1,
    }
}

/// Get a tensor's data as a float array handle
/// Only works for F32 tensors
/// Returns float array handle as i64 (pointer cast) on success, <=0 on error
#[no_mangle]
pub unsafe extern "C" fn gguf_get_tensor_f32(model_id: i64, name: *const c_char) -> i64 {
    if name.is_null() {
        return -1;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let models = LOADED_MODELS.lock().unwrap();
    let model = match models.get(&model_id) {
        Some(m) => m,
        None => return -2,
    };

    let tensor = match model.tensors.get(name_str) {
        Some(t) => t,
        None => return -3,
    };

    if tensor.dtype != GGML_TYPE_F32 {
        return -4; // Not F32
    }

    let numel: usize = tensor.dims.iter().product::<u64>() as usize;
    let data_start = model.tensor_data_offset + tensor.offset as usize;
    let data_end = data_start + numel * 4;

    if data_end > model.mmap.len() {
        return -5; // Out of bounds
    }

    // Create float array and copy data (converting f32 to f64)
    let arr_ptr = crate::float_array_create(numel as c_int);
    if arr_ptr.is_null() {
        return -6;
    }

    let mut current_ptr = arr_ptr;
    for i in 0..numel {
        let offset = data_start + i * 4;
        let value_f32 = read_f32(&model.mmap[..], offset);
        let value_f64 = value_f32 as f64; // Convert f32 to f64
        current_ptr = crate::float_array_push(current_ptr, value_f64);
    }

    current_ptr as i64
}

/// Get tensor dimension count
#[no_mangle]
pub unsafe extern "C" fn gguf_tensor_ndim(model_id: i64, name: *const c_char) -> c_int {
    if name.is_null() {
        return -1;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let models = LOADED_MODELS.lock().unwrap();
    match models.get(&model_id) {
        Some(model) => match model.tensors.get(name_str) {
            Some(tensor) => tensor.dims.len() as c_int,
            None => -1,
        },
        None => -1,
    }
}

/// Get tensor dimension at index
#[no_mangle]
pub unsafe extern "C" fn gguf_tensor_dim(model_id: i64, name: *const c_char, index: c_int) -> i64 {
    if name.is_null() || index < 0 {
        return -1;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let models = LOADED_MODELS.lock().unwrap();
    match models.get(&model_id) {
        Some(model) => match model.tensors.get(name_str) {
            Some(tensor) => {
                let idx = index as usize;
                if idx < tensor.dims.len() {
                    tensor.dims[idx] as i64
                } else {
                    -1
                }
            }
            None => -1,
        },
        None => -1,
    }
}

/// Check if a tensor exists
#[no_mangle]
pub unsafe extern "C" fn gguf_has_tensor(model_id: i64, name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let models = LOADED_MODELS.lock().unwrap();
    match models.get(&model_id) {
        Some(model) => {
            if model.tensors.contains_key(name_str) {
                1
            } else {
                0
            }
        }
        None => 0,
    }
}
