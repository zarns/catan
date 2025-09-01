use candle_core as candle;
use candle_nn::VarMap;
use std::path::Path;

/// Placeholder loader that will later read safetensors from disk.
pub fn load_latest_weights_path() -> String {
    "models/latest.safetensors".to_string()
}

pub fn device_auto() -> candle::Device {
    // Prefer CUDA device 0, fallback to CPU gracefully
    match candle::Device::new_cuda(0) {
        Ok(d) => {
            println!("[device] using CUDA:0");
            d
        }
        Err(e) => {
            println!("[device] CUDA unavailable ({}); using CPU", e);
            candle::Device::Cpu
        }
    }
}

pub fn try_load(varmap: &mut VarMap, path: &str) -> bool {
    let p = Path::new(path);
    if !p.exists() {
        return false;
    }
    match varmap.load(p) {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn try_save(varmap: &VarMap, path: &str) -> bool {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    varmap.save(p).is_ok()
}


