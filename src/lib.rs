#![feature(variant_count)]
#![feature(array_chunks)]
#![feature(array_methods)]
#![feature(generic_arg_infer)]
#![feature(more_float_constants)]
#![feature(array_windows)]
#![feature(associated_type_bounds)]
#![feature(portable_simd)]
#![feature(new_uninit)]
#![feature(let_chains)]
#![feature(const_fn_floating_point_arithmetic)]
#![allow(long_running_const_eval)]

#![feature(generic_const_exprs)]

use core::f64::consts::LN_2;
use std::f64::EPSILON;
use std::f64::consts::TAU;
use std::sync::Arc;

use num::Float;
use real_time_fir_iir_filters::{iir::first::{FirstOrderFilter, Omega}, rtf::Rtf};
use vst::{prelude::*, plugin_main};

moddef::moddef!(
    flat(pub) mod {
        parameters,
        tube_stage,
        reverb
    },
    mod {
        util
    }
);

const CHANNEL_COUNT: usize = 2;

const LOG_MID: f64 = 0.1;

#[test]
fn calc_curve()
{
    const MID: f64 = 0.991;
    println!("CURVE = {}", -MID.log2())
}

//const TREBLE_CUT_CURVE: f64 = 0.15200309344504995;
const LOG_CURVE: f64 = 3.321928094887362;

const G_PRE: f64 = 1.0;
const G_POST: f64 = 1.0;
const F_TRANSFORMER: f64 = 20.0;

const TREBLE_F: f64 = 3000.0;
const BASS_F: f64 = 440.0;

struct ReverbPlugin
{
    pub param: Arc<ReverbParameters>,
    tube_stage: [TubeStage; CHANNEL_COUNT],
    filter_transformer: [[FirstOrderFilter<f64>; 2]; CHANNEL_COUNT],
    tone_stack: [[FirstOrderFilter<f64>; 2]; CHANNEL_COUNT],
    reverb: [FDNReverb; CHANNEL_COUNT],
    rate: f64
}

#[inline]
fn soft_clip_max(x: f64, d: f64) -> f64
{
    let x = x.max(d);
    x - 1.0/(EPSILON - d).exp() + 1.0/(x - d).exp()
}

#[inline]
fn soft_clip_min(x: f64, d: f64) -> f64
{
    let x = x.min(d);
    x + 1.0/(d - EPSILON).exp() - 1.0/(d - x).exp()
}

#[inline]
fn soft_clip(x: f64, min: f64, max: f64) -> f64
{
    let x = x.max(min).min(max);
    x - 1.0/(max - x).exp() + 1.0/(x - min).exp() + 1.0/(max - EPSILON).exp() - 1.0/(EPSILON - min).exp()
}

#[inline]
fn x2exp(x: f64) -> f64
{
    (x*(2.0/LN_2)).exp2()
}

impl ReverbPlugin
{
    fn process<F: Float>(&mut self, buffer: &mut AudioBuffer<F>)
    {
        let gain = self.param.gain.get() as f64;
        let wet = self.param.wet.get() as f64;
        let dry = self.param.dry.get() as f64;
        let prescence = self.param.prescence.get() as f64;
        let mids = self.param.mids.get() as f64;
        let mud = self.param.mud.get() as f64;

        for (
                (c, (input_channel, output_channel)),
                (((tube_stage, filter_transformer), tone_stack), reverb)
            ) in buffer.zip()
            .enumerate()
            .zip(self.tube_stage.iter_mut()
                .zip(self.filter_transformer.iter_mut())
                .zip(self.tone_stack.iter_mut())
                .zip(self.reverb.iter_mut())
            )
        {
            reverb.update(&self.param);

            for (input_sample, output_sample) in input_channel.into_iter()
                .zip(output_channel.into_iter())
            {
                let x = input_sample.to_f64().unwrap();

                let [z_rest, z_treble] = tone_stack[0].filter(self.rate, x);
                let [z_bass, z_mids] = tone_stack[1].filter(self.rate, z_rest);
                let mut z = z_bass*mud + z_mids*mids + z_treble*prescence;

                z = tube_stage.next(self.rate, z*G_PRE*gain);
                [_, z] = filter_transformer[0].filter(self.rate, z);

                z = reverb.next(self.rate, z);

                z = if z.is_nan() {0.0} else {soft_clip(z, -30.0, 30.0)};

                [_, z] = filter_transformer[1].filter(self.rate, z);
                z *= G_POST;
                let y = if z.is_nan() {0.0} else {soft_clip(z, -30.0, 30.0)};

                *output_sample = F::from(y*wet + x/LOG_MID*dry).unwrap();
            }
        }
    }
}

#[allow(deprecated)]
impl Plugin for ReverbPlugin
{
    fn new(_host: HostCallback) -> Self
    where
        Self: Sized
    {
        let param = ReverbParameters::default();

        let reverb = [(); _].map(|()| FDNReverb::new());

        ReverbPlugin {
            param: Arc::new(param),
            tube_stage: [TubeStage::new(); _],
            filter_transformer: [[FirstOrderFilter::new(Omega::new(TAU*F_TRANSFORMER)); _]; CHANNEL_COUNT],
            tone_stack: [[FirstOrderFilter::new(Omega::new(TAU*TREBLE_F)), FirstOrderFilter::new(Omega::new(TAU*BASS_F))]; _],
            reverb,
            rate: 44100.0
        }
    }

    fn get_tail_size(&self) -> isize
    {
        const TAIL_A: f64 = 0.0000009932959;
        const TAIL_B: f64 = 17.4476084084878;

        let length = (D as f64/5000.0)*self.param.length.get() as f64;
        let feedback = self.param.feedback.get() as f64;

        (self.rate*length*TAIL_A*(feedback*TAIL_B).exp()) as isize
    }

    fn get_info(&self) -> Info
    {
        Info {
            name: "FDNverb".to_string(),
            vendor: "Soma FX".to_string(),
            presets: 0,
            parameters: ReverbParam::VARIANT_COUNT as i32,
            inputs: CHANNEL_COUNT as i32,
            outputs: CHANNEL_COUNT as i32,
            midi_inputs: 0,
            midi_outputs: 0,
            unique_id: 1323532,
            version: 1,
            category: Category::Effect,
            initial_delay: 0,
            preset_chunks: false,
            f64_precision: true,
            silent_when_stopped: true,
            ..Default::default()
        }
    }

    fn set_sample_rate(&mut self, rate: f32)
    {
        self.rate = rate as f64;
    }

    fn resume(&mut self)
    {
        
    }

    fn suspend(&mut self)
    {
        for filters in self.filter_transformer.iter_mut()
        {
            for filter in filters.iter_mut()
            {
                filter.reset()
            }
        }
        for reverb in self.reverb.iter_mut()
        {
            reverb.suspend();
        }
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>)
    {
        self.process(buffer)
    }

    fn process_f64(&mut self, buffer: &mut AudioBuffer<f64>)
    {
        self.process(buffer)
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters>
    {
        self.param.clone()
    }
}

plugin_main!(ReverbPlugin);