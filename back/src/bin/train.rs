use env_logger;
use log::info;

fn main() {
    env_logger::init();
    info!(
        "Training entrypoint stub: implement self-play, replay buffer, and optimization next."
    );
    println!(
        "train.rs: Stub. Next steps: generate self-play, build mini ResNet in nn/model.rs, and save to models/latest.safetensors"
    );
}


