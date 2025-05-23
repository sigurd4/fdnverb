use core::f64::consts::TAU;

use real_time_fir_iir_filters::{conf::{All, HighPass}, filters::iir::first::FirstOrderFilter, param::{Omega, RC}, rtf::Rtf};
use saturation::{tubes::Tube12AU7, Triode, TriodeClassA};

use crate::{reverb::M, FDNReverb, ReverbParameters, LOG_MID};

pub const HEADROOM: f64 = 3.0;
const G_PRE: f64 = 1.0/HEADROOM;
const G_POST: f64 = 2.0*HEADROOM;
const F_TRANSFORMER: f64 = 20.0;

const TREBLE_F: f64 = 3000.0;
const BASS_F: f64 = 440.0;

#[derive(Debug, Clone)]
pub struct Channel
{
    filter_transformer: [FirstOrderFilter<HighPass, f64>; 2],
    tone_stack: [FirstOrderFilter<All, f64>; 2],
    tube1: Triode<f64, Tube12AU7, ()>,
    tube2: Triode<f64, Tube12AU7>,
    reverb: FDNReverb,
    x: f64,
    z: f64,
}

impl Channel
{
    pub fn process1(&mut self, rate: f64, x: f64, gain: f64, mud: f64, mids: f64, prescence: f64) -> &[f64; M]
    {
        self.x = x;
        let [z_rest, z_treble] = self.tone_stack[0].filter(rate, self.x);
        let [z_bass, z_mids] = self.tone_stack[1].filter(rate, z_rest);
        self.z = z_bass*mud + z_mids*mids + z_treble*prescence;

        self.z *= G_PRE*gain;
        self.z = self.tube1.saturate(rate, self.z);
        [self.z] = self.filter_transformer[0].filter(rate, self.z);

        self.reverb.process1(rate)
    }

    pub fn process2(&mut self, rate: f64, z_avg: &[f64; M], wet: f64, dry: f64, stereo_separation: f64) -> f64
    {
        self.z = self.reverb.process2(self.z, z_avg, stereo_separation);

        [self.z] = self.filter_transformer[1].filter(rate, self.z);
        self.z *= G_POST;
        self.z = self.tube2.saturate(rate, self.z);

        self.z*wet + self.x/LOG_MID*dry
    }

    pub fn update(&mut self, params: &ReverbParameters)
    {
        self.reverb.update(params);
    }

    pub fn suspend(&mut self)
    {
        for filter in self.filter_transformer.iter_mut()
        {
            filter.reset();
        }
        self.reverb.suspend();
    }
}

impl Default for Channel
{
    fn default() -> Self
    {
        const DY: f64 = 0.05;
        const R_T: f64 = (458.0 - 440.0)/(8.6/2.2e3);
        Self {
            filter_transformer: [
                FirstOrderFilter::new(Omega {
                    omega: TAU*F_TRANSFORMER
                });
                2
            ],
            tone_stack: [
                FirstOrderFilter::new(Omega {
                    omega: TAU*TREBLE_F
                }),
                FirstOrderFilter::new(Omega {
                    omega: TAU*BASS_F
                })
            ],
            tube1: Triode::new(
                TriodeClassA {
                    r_i: 0.0,
                    r_p: R_T,
                    v_pp: 458.0,
                    v_c: 8.6
                }.cache(DY),
                Default::default(),
                RC {
                    r: 2.2e3*2.0,
                    c: 25e-6
                }
            ),
            tube2: Triode::new(
                TriodeClassA {
                    r_i: R_T + 47e3,
                    r_p: 100e3,
                    v_pp: 410.0,
                    v_c: 2.0
                }.cache(DY),
                Default::default(),
                RC {
                    r: 820.0,
                    c: 25e-6
                }
            ),
            reverb: FDNReverb::new(),
            x: 0.0,
            z: 0.0
        }
    }   
}