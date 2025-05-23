#![feature(variant_count)]
#![feature(array_chunks)]
#![feature(generic_arg_infer)]
#![feature(more_float_constants)]
#![feature(array_windows)]
#![feature(portable_simd)]
#![feature(let_chains)]
#![allow(long_running_const_eval)]
#![feature(iter_array_chunks)]
#![feature(allocator_api)]
#![feature(impl_trait_in_bindings)]
#![feature(future_join)]
#![feature(array_try_map)]
#![feature(generic_const_exprs)]

use std::sync::Arc;

use num::Float;
use vst::{prelude::*, plugin_main};

moddef::moddef!(
    flat mod {
        bank,
        channel,
        parameters,
        reverb
    },
    mod {
        util
    }
);

const CHANNEL_COUNT: usize = 2;

const EQ_MAX: f32 = 10.0;
const LOG_MID: f64 = 0.1;

#[cfg(test)]
#[test]
fn calc_curve()
{
    const MID: f64 = 0.991;
    println!("CURVE = {}", -MID.log2())
}

#[cfg(test)]
#[test]
fn it_works()
{
    let rate = 44100.0;
    let x = 1.0;

    let param = ReverbParameters::default();

    let mut c = Channel::default();
    c.update(&param);
    
    let z_avg = *c.process1(rate, x, 0.5, 0.5, 0.5, 0.5);
    c.process2(rate, &z_avg, 0.5, 0.5, 0.5);

    let z_avg = *c.process1(rate, x, 0.5, 0.5, 0.5, 0.5);
    c.process2(rate, &z_avg, 0.5, 0.5, 0.5);
}

//const TREBLE_CUT_CURVE: f64 = 0.15200309344504995;
const LOG_CURVE: f64 = 3.321928094887362;

struct ReverbPlugin
{
    pub param: Arc<ReverbParameters>,
    channels: [Channel; CHANNEL_COUNT],
    rate: f64
}

impl ReverbPlugin
{
    fn process<'a, F: Float>(&mut self, buffer: &mut AudioBuffer<'a, F>)
    {
        let gain = self.param.gain.get() as f64;
        let wet = self.param.wet.get() as f64;
        let dry = self.param.dry.get() as f64;
        let stereo_separation = self.param.stereo_separation.get() as f64;
        let stereo_merging = 1.0 - stereo_separation;
        let prescence = self.param.prescence.get() as f64;
        let mids = self.param.mids.get() as f64;
        let mud = self.param.mud.get() as f64;

        for channel in self.channels.iter_mut()
        {
            channel.update(&self.param);
        }

        let (input_buffer, mut output_buffer) = buffer.split();

        let mut input = input_buffer.into_iter().map(|i| i.iter()).array_chunks::<CHANNEL_COUNT>().next().unwrap();
        let mut output = output_buffer.into_iter().map(|o| o.iter_mut()).array_chunks::<CHANNEL_COUNT>().next().unwrap();

        let input = input.each_mut();
        let output = output.each_mut();

        while let (Some(x), Some(y)) = (
            input.try_map(|input| input.next()),
            output.try_map(|output| output.next())
        )
        {
            let mut z_avg = [0.0; M];

            for (x, channel) in x.into_iter()
                .zip(self.channels.iter_mut())
            {
                let x = x.to_f64().unwrap();
                let z = channel.process1(self.rate, x, gain, mud, mids, prescence);
                for (z_avg, z) in z_avg.iter_mut()
                    .zip(z)
                {
                    *z_avg += *z;
                }
            }

            {
                let a = stereo_merging/CHANNEL_COUNT as f64;
                for z_avg in z_avg.iter_mut()
                {
                    *z_avg *= a
                }
            }

            for (y, channel) in y.into_iter()
                .zip(self.channels.iter_mut())
            {
                *y = F::from(channel.process2(self.rate, &z_avg, wet, dry, stereo_separation)).unwrap()
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
        ReverbPlugin {
            /*runtime: tokio::runtime::Builder::new_current_thread()
                .global_queue_interval(128)
                .worker_threads(2)
                .build()
                .expect("Invalid runtime."),*/
            param: Default::default(),
            channels: Default::default(),
            rate: 44100.0
        }
    }

    fn get_tail_size(&self) -> isize
    {
        // Magic numbers... teehee
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
            silent_when_stopped: true
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
        for channel in self.channels.iter_mut()
        {
            channel.suspend()
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