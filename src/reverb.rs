use std::{f64::consts::TAU, sync::atomic::Ordering};

use num::traits::MulAddAssign;
use real_time_fir_iir_filters::{conf::{All, LowPass}, filters::iir::first::FirstOrderFilter, param::Omega, rtf::Rtf};
use delay_line::DelayLine;

use crate::{util, ReverbParameters};

pub const M: usize = 32;
pub const D: usize = 50000;

pub const PHASES: [[f64; M]; M*M] = util::phases();

//pub const Q: [[f64; M]; M] = util::hadamard_feedback_matrix();

#[cfg(test)]
#[test]
fn primes()
{
    println!("{:?}", util::primes::<M>(100, 3));
}

#[derive(Debug, Clone)]
pub struct FDNReverb
{
    w: [DelayLine<f64>; M],
    f_f: [FirstOrderFilter<All, f64>; M],
    f_c: [FirstOrderFilter<LowPass, f64>; M],
    prime_curve: f64,
    length: f64,
    phase: u16,
    feedback: f64,
    p: [usize; M],
    g: [f64; M],
    q: [[f64; M]; M],
    z: [f64; M]
}

impl FDNReverb
{
    pub fn new() -> Self
    {
        Self {
            w: [(); _].map(|()| DelayLine::new()),
            f_f: [
                FirstOrderFilter::new(Omega {
                    omega: TAU*220.0
                });
                _
            ],
            f_c: [
                FirstOrderFilter::new(Omega {
                    omega: TAU*220.0
                });
                _
            ],
            prime_curve: 0.0,
            length: 0.0,
            phase: 0,
            feedback: 0.0,
            p: [0; _],
            g: [1.0; _],
            q: util::hadamard_feedback_matrix(0.0),
            z: [0.0; _]
        }
    }

    pub fn update(&mut self, params: &ReverbParameters)
    {
        // Update feedback gains
        let feedback = params.feedback.get() as f64;
        let phase = params.phase.load(Ordering::Relaxed);
        if self.phase != phase || self.feedback != feedback
        {
            self.phase = phase;
            self.feedback = feedback;
            self.g = PHASES[self.phase as usize];
            for g in self.g.iter_mut()
            {
                *g *= feedback;
            }
        }

        // Update feedback matrix
        /*let kernel = params.kernel.get() as f64;
        if self.kernel != kernel
        {
            self.kernel = kernel;
            self.q = util::hadamard_feedback_matrix(kernel)
        }*/

        // Update filters
        let floor = params.floor.get() as f64*TAU;
        let ceiling = params.ceiling.get() as f64*TAU;
        if self.f_f[0].param.omega != floor
        {
            for filter in self.f_f.iter_mut()
            {
                filter.param.omega = floor;
            }
        }
        if self.f_c[0].param.omega != ceiling
        {
            for filter in self.f_c.iter_mut()
            {
                filter.param.omega = ceiling;
            }
        }

        // Update delay lines
        let prime_curve = params.primes.get() as f64;
        let length = params.length.get() as f64;
        if prime_curve != self.prime_curve || length != self.length
        {
            self.p = util::primes_dist(prime_curve, D as f64*length);
            for (w, &p) in self.w.iter_mut()
                .zip(self.p.iter())
            {
                w.stretch(p);
            }

            self.prime_curve = prime_curve;
            self.length = length;
        }
    }

    pub fn process1(&mut self, rate: f64) -> &[f64; M]
    {
        for ((z, w), (f_f, f_c)) in self.z.iter_mut()
            .zip(self.w.iter())
            .zip(self.f_f.iter_mut()
                .zip(self.f_c.iter_mut())
            )
        {
            *z = w.output()
                .copied()
                .unwrap_or(0.0);

            // Apply feedback
            let b = f_f.param.omega > f_c.param.omega;
            let z_h;
            [*z, z_h] = {
                let zz = f_f.filter(rate, *z);
                [zz[!b as usize], zz[b as usize]]
            };
            [*z] = f_c.filter(rate, *z);
            if b
            {
                *z += z_h
            }
        }

        util::rmul_matrix_assign_row(&self.q, &mut self.z);

        &self.z
    }

    pub fn process2(&mut self, x: f64, z_avg: &[f64; M], stereo_separation: f64) -> f64
    {
        let mut y = 0.0;

        for (((z, &z_avg), &g), w) in self.z.iter_mut()
            .zip(z_avg)
            .zip(self.g.iter())
            .zip(self.w.iter_mut())
        {
            z.mul_add_assign(stereo_separation, z_avg);
            z.mul_add_assign(g, x);
            
            y += w.delay(*z);
        }

        y
    }

    pub fn suspend(&mut self)
    {
        for w in self.w.each_mut()
        {
            w.fill(0.0);
        }
    }
}