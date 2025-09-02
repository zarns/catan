use candle_core as candle;
use candle_core::{DType, Result, Tensor};
use candle_nn as nn;
use candle_nn::{Conv2d, Conv2dConfig, Linear, Module, VarBuilder, VarMap};

pub const ACTION_FEAT_DIM: usize = 8;

/// Minimal model placeholders; to be filled with a real ResNet.
pub struct ResBlock {
    conv1: Conv2d,
    conv2: Conv2d,
}

impl ResBlock {
    pub fn new(vb: &VarBuilder, channels: usize) -> Result<Self> {
        let cfg = Conv2dConfig {
            padding: 1,
            ..Default::default()
        };
        let conv1 = nn::conv2d(channels, channels, 3, cfg, vb.pp("conv1"))?;
        let conv2 = nn::conv2d(channels, channels, 3, cfg, vb.pp("conv2"))?;
        Ok(Self { conv1, conv2 })
    }
}

impl Module for ResBlock {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let residual = xs;
        let xs = self.conv1.forward(xs)?.relu()?;
        let xs = self.conv2.forward(&xs)?;
        (xs + residual)?.relu()
    }
}

pub struct AlphaZeroNet {
    pub device: candle::Device,
    pub varmap: VarMap,
    conv1: Conv2d,
    tower: Vec<ResBlock>,
    // Policy head
    policy_conv: Conv2d,
    policy_fc: Linear,
    policy_action_fc: Linear,
    // Value head
    value_fc1: Linear,
    value_fc2: Linear,
}

impl AlphaZeroNet {
    pub fn new(device: candle::Device) -> Result<Self> {
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        // 23 input channels → 32
        let conv_cfg = Conv2dConfig {
            padding: 1,
            ..Default::default()
        };
        let conv1 = nn::conv2d(23, 32, 3, conv_cfg, vb.pp("conv1"))?;

        // Small residual tower
        let mut tower = Vec::new();
        for i in 0..2 {
            tower.push(ResBlock::new(&vb.pp(format!("res{i}")), 32)?);
        }

        // Policy head: 1x1 conv to 2 channels, then global average pooling -> fc to placeholder (1)
        let policy_conv = nn::conv2d(32, 2, 1, Default::default(), vb.pp("policy_conv"))?;
        let policy_fc = nn::linear(2, 1, vb.pp("policy_fc"))?;
        // Action-conditional policy: Linear over [state_embed(32) || action_feat(8)] -> 1 logit per action
        let policy_action_fc = nn::linear(32 + ACTION_FEAT_DIM, 1, vb.pp("policy_action_fc"))?;

        // Value head: GAP -> 64 -> 1 (tanh)
        let value_fc1 = nn::linear(32, 64, vb.pp("value_fc1"))?;
        let value_fc2 = nn::linear(64, 1, vb.pp("value_fc2"))?;

        Ok(Self {
            device,
            varmap,
            conv1,
            tower,
            policy_conv,
            policy_fc,
            policy_action_fc,
            value_fc1,
            value_fc2,
        })
    }

    /// Forward: xs shape [C,H,W]; returns (policy_logits dummy, value)
    pub fn forward(&self, xs: &Tensor) -> Result<(Tensor, Tensor)> {
        // Add batch dimension: [1,C,H,W]
        let xs = xs.unsqueeze(0)?;
        let xs = self.conv1.forward(&xs)?.relu()?;
        let mut xs = xs;
        for block in &self.tower {
            xs = block.forward(&xs)?;
        }
        // Policy branch
        let p = self.policy_conv.forward(&xs)?.relu()?;
        let p = p.mean(2)?; // [1,2,W]
        let p = p.mean(2)?; // [1,2]
        let policy_logits = self.policy_fc.forward(&p)?; // [1, actions] (placeholder: 1)

        // Value branch
        let xs = xs.mean(2)?; // [1,C,W]
        let xs = xs.mean(2)?; // [1,C]

        let v = self.value_fc1.forward(&xs)?.relu()?;
        let v = self.value_fc2.forward(&v)?.tanh()?; // [1,1]

        Ok((policy_logits, v))
    }

    /// Returns (state_embed [1,32], value [1,1]) from input [C,H,W]
    pub fn forward_embed(&self, xs: &Tensor) -> Result<(Tensor, Tensor)> {
        let xs = xs.unsqueeze(0)?;
        let xs = self.conv1.forward(&xs)?.relu()?;
        let mut xs = xs;
        for block in &self.tower {
            xs = block.forward(&xs)?;
        }
        // Policy branch not needed here
        // Value branch: also return the pooled features as state embedding
        let pooled = xs.mean(2)?.mean(2)?; // [1,32]
        let v = self.value_fc1.forward(&pooled)?.relu()?;
        let v = self.value_fc2.forward(&v)?.tanh()?; // [1,1]
        Ok((pooled, v))
    }

    /// Batched variant: input [B,C,H,W] → (embeds [B,32], value [B,1])
    pub fn forward_embed_batch(&self, xs: &Tensor) -> Result<(Tensor, Tensor)> {
        // xs: [B,C,H,W]
        let xs = self.conv1.forward(xs)?.relu()?;
        let mut xs = xs;
        for block in &self.tower {
            xs = block.forward(&xs)?;
        }
        let pooled = xs.mean(2)?.mean(2)?; // [B,32]
        let v = self.value_fc1.forward(&pooled)?.relu()?;
        let v = self.value_fc2.forward(&v)?.tanh()?; // [B,1]
        Ok((pooled, v))
    }

    /// Compute per-action logits from fused input [K, 32 + ACTION_FEAT_DIM] -> [K,1]
    pub fn policy_logits(&self, fused: &Tensor) -> Result<Tensor> {
        self.policy_action_fc.forward(fused)
    }
}
