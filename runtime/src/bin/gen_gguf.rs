//! Synthetic GGUF Generator
//! Creates a minimal LLaMA-style model for testing the inference pipeline

use rand::Rng;
use std::fs::File;
use std::io::{BufWriter, Seek, Write};

// GGUF constants
const GGUF_MAGIC: u32 = 0x46554747; // "GGUF" in little-endian
const GGUF_VERSION: u32 = 3;
const GGML_TYPE_F32: u32 = 0;

// Metadata value types
const GGUF_TYPE_UINT32: u32 = 4;
const GGUF_TYPE_STRING: u32 = 8;

// Model configuration
struct ModelConfig {
    vocab_size: usize,
    hidden_dim: usize,
    intermediate_dim: usize,
    num_layers: usize,
    num_heads: usize,
    max_seq_len: usize,
}

impl ModelConfig {
    fn tiny() -> Self {
        Self {
            vocab_size: 256,      // Small vocab for testing
            hidden_dim: 64,       // Tiny hidden dimension
            intermediate_dim: 256, // 4x hidden_dim
            num_layers: 2,        // Just 2 layers
            num_heads: 4,         // 4 attention heads
            max_seq_len: 128,     // Short context
        }
    }
}

struct TensorInfo {
    name: String,
    dims: Vec<u64>,
    dtype: u32,
    offset: u64,
}

fn write_string(writer: &mut impl Write, s: &str) -> std::io::Result<()> {
    let bytes = s.as_bytes();
    writer.write_all(&(bytes.len() as u64).to_le_bytes())?;
    writer.write_all(bytes)?;
    Ok(())
}

fn write_metadata_string(writer: &mut impl Write, key: &str, value: &str) -> std::io::Result<()> {
    write_string(writer, key)?;
    writer.write_all(&GGUF_TYPE_STRING.to_le_bytes())?;
    write_string(writer, value)?;
    Ok(())
}

fn write_metadata_u32(writer: &mut impl Write, key: &str, value: u32) -> std::io::Result<()> {
    write_string(writer, key)?;
    writer.write_all(&GGUF_TYPE_UINT32.to_le_bytes())?;
    writer.write_all(&value.to_le_bytes())?;
    Ok(())
}

fn generate_random_weights(size: usize) -> Vec<f32> {
    let mut rng = rand::thread_rng();
    let scale = 0.02; // Small initialization
    (0..size).map(|_| rng.gen_range(-scale..scale) as f32).collect()
}

fn main() -> std::io::Result<()> {
    let config = ModelConfig::tiny();
    let output_path = std::env::args().nth(1).unwrap_or_else(|| "tiny_model.gguf".to_string());

    println!("Generating synthetic GGUF model:");
    println!("  Vocab size: {}", config.vocab_size);
    println!("  Hidden dim: {}", config.hidden_dim);
    println!("  Layers: {}", config.num_layers);
    println!("  Heads: {}", config.num_heads);
    println!("  Output: {}", output_path);

    // Build tensor list
    let mut tensors: Vec<TensorInfo> = Vec::new();
    let mut current_offset: u64 = 0;

    // Token embeddings: [vocab_size, hidden_dim]
    let embed_size = config.vocab_size * config.hidden_dim;
    tensors.push(TensorInfo {
        name: "token_embd.weight".to_string(),
        dims: vec![config.hidden_dim as u64, config.vocab_size as u64],
        dtype: GGML_TYPE_F32,
        offset: current_offset,
    });
    current_offset += (embed_size * 4) as u64;

    // Per-layer tensors
    for layer in 0..config.num_layers {
        // Attention norm (RMSNorm): [hidden_dim]
        tensors.push(TensorInfo {
            name: format!("blk.{}.attn_norm.weight", layer),
            dims: vec![config.hidden_dim as u64],
            dtype: GGML_TYPE_F32,
            offset: current_offset,
        });
        current_offset += (config.hidden_dim * 4) as u64;

        // Q, K, V, O projections: [hidden_dim, hidden_dim] each
        for proj in &["attn_q", "attn_k", "attn_v", "attn_output"] {
            let size = config.hidden_dim * config.hidden_dim;
            tensors.push(TensorInfo {
                name: format!("blk.{}.{}.weight", layer, proj),
                dims: vec![config.hidden_dim as u64, config.hidden_dim as u64],
                dtype: GGML_TYPE_F32,
                offset: current_offset,
            });
            current_offset += (size * 4) as u64;
        }

        // FFN norm (RMSNorm): [hidden_dim]
        tensors.push(TensorInfo {
            name: format!("blk.{}.ffn_norm.weight", layer),
            dims: vec![config.hidden_dim as u64],
            dtype: GGML_TYPE_F32,
            offset: current_offset,
        });
        current_offset += (config.hidden_dim * 4) as u64;

        // FFN gate, up: [intermediate_dim, hidden_dim]
        for proj in &["ffn_gate", "ffn_up"] {
            let size = config.intermediate_dim * config.hidden_dim;
            tensors.push(TensorInfo {
                name: format!("blk.{}.{}.weight", layer, proj),
                dims: vec![config.hidden_dim as u64, config.intermediate_dim as u64],
                dtype: GGML_TYPE_F32,
                offset: current_offset,
            });
            current_offset += (size * 4) as u64;
        }

        // FFN down: [hidden_dim, intermediate_dim]
        let size = config.hidden_dim * config.intermediate_dim;
        tensors.push(TensorInfo {
            name: format!("blk.{}.ffn_down.weight", layer),
            dims: vec![config.intermediate_dim as u64, config.hidden_dim as u64],
            dtype: GGML_TYPE_F32,
            offset: current_offset,
        });
        current_offset += (size * 4) as u64;
    }

    // Output norm: [hidden_dim]
    tensors.push(TensorInfo {
        name: "output_norm.weight".to_string(),
        dims: vec![config.hidden_dim as u64],
        dtype: GGML_TYPE_F32,
        offset: current_offset,
    });
    current_offset += (config.hidden_dim * 4) as u64;

    // Output projection: [vocab_size, hidden_dim]
    let output_size = config.vocab_size * config.hidden_dim;
    tensors.push(TensorInfo {
        name: "output.weight".to_string(),
        dims: vec![config.hidden_dim as u64, config.vocab_size as u64],
        dtype: GGML_TYPE_F32,
        offset: current_offset,
    });
    current_offset += (output_size * 4) as u64;

    let total_tensor_bytes = current_offset;
    println!("  Total tensors: {}", tensors.len());
    println!("  Total tensor data: {} bytes ({:.2} MB)", total_tensor_bytes, total_tensor_bytes as f64 / 1024.0 / 1024.0);

    // Metadata - store string values to avoid lifetime issues
    let context_len_str = config.max_seq_len.to_string();
    let hidden_dim_str = config.hidden_dim.to_string();
    let num_layers_str = config.num_layers.to_string();
    let num_heads_str = config.num_heads.to_string();
    let intermediate_str = config.intermediate_dim.to_string();

    let metadata = vec![
        ("general.architecture", "llama"),
        ("general.name", "TinySyntheticLlama"),
        ("llama.context_length", context_len_str.as_str()),
        ("llama.embedding_length", hidden_dim_str.as_str()),
        ("llama.block_count", num_layers_str.as_str()),
        ("llama.attention.head_count", num_heads_str.as_str()),
        ("llama.feed_forward_length", intermediate_str.as_str()),
    ];

    // Write GGUF file
    let file = File::create(&output_path)?;
    let mut writer = BufWriter::new(file);

    // Header
    writer.write_all(&GGUF_MAGIC.to_le_bytes())?;
    writer.write_all(&GGUF_VERSION.to_le_bytes())?;
    writer.write_all(&(tensors.len() as u64).to_le_bytes())?;
    writer.write_all(&(metadata.len() as u64).to_le_bytes())?;

    // Metadata
    for (key, value) in &metadata {
        if key.contains("length") || key.contains("count") {
            let v: u32 = value.parse().unwrap();
            write_metadata_u32(&mut writer, key, v)?;
        } else {
            write_metadata_string(&mut writer, key, value)?;
        }
    }

    // Tensor infos
    for tensor in &tensors {
        write_string(&mut writer, &tensor.name)?;
        writer.write_all(&(tensor.dims.len() as u32).to_le_bytes())?;
        for dim in &tensor.dims {
            writer.write_all(&dim.to_le_bytes())?;
        }
        writer.write_all(&tensor.dtype.to_le_bytes())?;
        writer.write_all(&tensor.offset.to_le_bytes())?;
    }

    // Pad to 32-byte alignment
    let current_pos = writer.stream_position().unwrap_or(0);
    let padding = (32 - (current_pos % 32)) % 32;
    for _ in 0..padding {
        writer.write_all(&[0u8])?;
    }

    // Write tensor data
    println!("Generating random weights...");
    for tensor in &tensors {
        let num_elements: usize = tensor.dims.iter().map(|&d| d as usize).product();
        let weights = generate_random_weights(num_elements);
        for w in &weights {
            writer.write_all(&w.to_le_bytes())?;
        }
    }

    writer.flush()?;
    println!("Done! Created {}", output_path);

    Ok(())
}
