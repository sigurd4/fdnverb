use std::f64::consts::TAU;

use real_time_fir_iir_filters::{iir::first::{FirstOrderFilter, Omega}, rtf::Rtf};

const G_CLIP: f64 = 0.1;
const P_HARD_CLIP: f64 = 0.2;
const P_HARD_CLIP_FINAL: f64 = 0.8;
const FILTER_TUBE_CASCADE: usize = 9;
const FILTER_TUBE_FREQUENCIES: [f64; FILTER_TUBE_CASCADE] = [
    22.0,
    47.0,
    100.0,
    220.0,
    470.0,
    1000.0,
    2200.0,
    4700.0,
    10000.0
];
const G_OVERTONES: f64 = 1.0;
const P_SOFT_CLIP: f64 = 1.0;
const G_SOFT_CLIP: f64 = 1.13;
const P_SIGMOID_CLIP: f64 = 0.99;

#[derive(Clone, Copy)]
pub struct TubeStage
{
    filter_tube: [FirstOrderFilter<f64>; FILTER_TUBE_CASCADE],
}

impl TubeStage
{
    pub fn new() -> Self
    {
        Self {
            filter_tube: FILTER_TUBE_FREQUENCIES.map(|f| FirstOrderFilter::new(Omega::new(TAU*f)))
        }
    }

    pub fn next(&mut self, rate: f64, mut x: f64) -> f64
    {
        x = crate::soft_clip(x, -200.0, 140.0)*P_HARD_CLIP_FINAL + (1.0 - P_HARD_CLIP_FINAL)*x;

        for filter_tube in self.filter_tube.iter_mut()
        {
            let [z1, z2] = filter_tube.filter(rate, x);

            x = z2*G_OVERTONES - z1*P_SOFT_CLIP;
            x *= G_CLIP;
            let x2exp = crate::x2exp(x);
            x = P_SIGMOID_CLIP*(x2exp - 1.0)/(1.0 + x2exp) + (1.0 - P_SIGMOID_CLIP)*x;
            x /= G_CLIP;
            x = crate::soft_clip(x, -2.0, 1.4)*P_HARD_CLIP + (1.0 - P_HARD_CLIP)*x;

            x = z1*(1.0 + P_SOFT_CLIP*G_SOFT_CLIP) + x*G_SOFT_CLIP;
        }
        
        x *= G_CLIP;
        let x2exp = crate::x2exp(x);
        x = P_SIGMOID_CLIP*(x2exp - 1.0)/(1.0 + x2exp) + (1.0 - P_SIGMOID_CLIP)*x;
        x /= G_CLIP;
        x = crate::soft_clip(x, -2.0, 1.4)*P_HARD_CLIP + (1.0 - P_HARD_CLIP)*x;

        x
    }
}