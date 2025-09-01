# Catan

## front

- run `ng serve`
- to deploy:
  - from front dir
  - `ng build --configuration production` (builds with production settings)
  - from root dir
  - `firebase login`
  - `firebase deploy`

## back

- from back dir
- run `shuttle run`
- deploy `shuttle deploy`

## simulation

The project includes a CLI tool for simulating Catan games between AI players:

- Build: `cargo build --bin simulate`
- Run: `cargo run --bin simulate -- [OPTIONS]`
- Run (optimized): `cargo run --release --bin simulate -- [OPTIONS]`

### Options

- `-p, --players <CONFIG>`: Player types (e.g., "MR" for MCTS vs Random)
  - `R`: Random player
  - `M`: MCTS player
- `-n, --num_games <N>`: Number of games to simulate (default: 1)
- `-v, --verbose`: Show detailed game logs

### Examples

- MCTS vs Random (single game): `cargo run --bin simulate -- -p MR`
- Random vs MCTS (10 games): `cargo run --bin simulate -- -p RM -n 10`
- Random vs Random with logs: `cargo run --bin simulate -- -p RR -v`

## Attribution

Inspired by [bcollazo's Catanatron](https://github.com/bcollazo/catanatron). Licensed under GPL-3.0.


# TODO
- Build the greatest catan bot player of all time
- Implement DB functionality to track user count and game history
  - MCP integration
- Robber getting moved after first turn to wood 10 tile every time?


wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update
sudo apt-get -y install cuda-toolkit-12-5


## WSL2 GPU (CUDA) setup for Candle

Follow these steps inside your WSL2 Ubuntu terminal to enable CUDA for Candle.

1) Install CUDA toolkit in WSL2 (Ubuntu 24.04)

```bash
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update
sudo apt-get -y install cuda-toolkit-12-5
```

2) Point canonical symlink to the installed version (12.5)

```bash
sudo ln -sfn /usr/local/cuda-12.5 /usr/local/cuda
```

3) Add CUDA to PATH and runtime libraries (make persistent)

```bash
echo 'export PATH=/usr/local/cuda/bin:$PATH' >> ~/.bashrc
echo 'export LD_LIBRARY_PATH=/usr/local/cuda/lib64:/usr/lib/wsl/lib:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc
```

4) Verify GPU visibility in WSL

```bash
nvidia-smi
which nvcc && nvcc --version
ls -l /usr/lib/wsl/lib/libcuda.so.1
```

5) Enable Candle CUDA features (already done in this repo)

```toml
# back/Cargo.toml
[dependencies]
candle-core = { version = "0.9.1", features = ["cuda"] }
candle-nn   = { version = "0.9.1", features = ["cuda"] }
```

6) Rebuild and run

```bash
cargo clean
cargo run --bin train
```

Expected logs:
- `[device] using CUDA:0`
- `[infer] device=Device(Cuda(0))`

### Troubleshooting

- Do not set `CUDARC_CUDA_VERSION`.
  - If you previously exported it, unset and rebuild so cudarc can auto-detect from `nvcc`:
    ```bash
    unset CUDARC_CUDA_VERSION
    cargo clean && cargo run --bin train
    ```

- Still seeing CPU device in logs
  - Confirm WSL2 usage and `nvidia-smi` works inside WSL.
  - Ensure `/usr/lib/wsl/lib/libcuda.so.1` exists and `LD_LIBRARY_PATH` includes `/usr/lib/wsl/lib`.
  - Re-run with a fresh shell or `source ~/.bashrc`.

- Build is slow while testing
  - Use a short training run: `SELF_PLAY_GAMES=1` (default here).
  - Try optimized build: `cargo run --release --bin train`.
