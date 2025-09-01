use candle_core as candle;

/// Placeholder loader that will later read safetensors from disk.
pub fn load_latest_weights_path() -> String {
    "models/latest.safetensors".to_string()
}

pub fn device_auto() -> candle::Device {
    match candle::Device::new_cuda(0) {
        Ok(d) => d,
        Err(_) => candle::Device::Cpu,
    }
}


