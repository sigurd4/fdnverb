use core::mem::MaybeUninit;
use std::{f64::consts::TAU, sync::atomic::Ordering};

use array_math::{ArrayOps, MatrixMath, SliceOps};
use num::traits::MulAddAssign;
use real_time_fir_iir_filters::{iir::first::{FirstOrderFilter, Omega}, rtf::Rtf};

use crate::{util, ReverbParameters};

pub const M: usize = 16;
pub const D: usize = 5000;

pub const PHASES: [[f64; M]; M*M] = util::phases();

//pub const Q: [[f64; M]; M] = util::hadamard_feedback_matrix();

#[cfg(test)]
#[test]
fn primes()
{
    println!("{:?}", util::primes::<M>(100, 3));
}

#[derive(Clone)]
pub struct FDNReverb
{
    w: [Vec<f64>; M],
    f: [[FirstOrderFilter<f64>; 2]; M],
    prime_curve: f64,
    length: f64,
    //b: [f64; M],
    //c: [f64; M],
    g: [f64; M],
    q: [[f64; M]; M]
}

impl FDNReverb
{
    pub fn new() -> Self
    {
        Self {
            w: [(); _].map(|()| vec![]),
            f: [[FirstOrderFilter::new(Omega::new(TAU*220.0)); _]; _],
            prime_curve: 0.0,
            length: 0.0,
            //b: [1.0; _],
            //c: [1.0; _],
            g: [1.0; _],
            q: util::hadamard_feedback_matrix()
        }
    }

    pub fn update(&mut self, params: &ReverbParameters)
    {
        let feedback = params.feedback.get() as f64;
        let phase = params.phase.load(Ordering::Relaxed) as usize;
        self.g = PHASES[phase].mul_all(feedback);
        let floor = params.floor.get() as f64;
        let ceiling = params.ceiling.get() as f64;
        for [filter_f, filter_c] in self.f.each_mut()
        {
            filter_f.param.omega.assign(TAU*floor);
            filter_c.param.omega.assign(TAU*ceiling);
        }
        let prime_curve = params.primes.get() as f64;
        let length = params.length.get() as f64;
        if prime_curve != self.prime_curve || length != self.length
        {
            let p = util::primes_dist(prime_curve, D as f64*length);
            for (w, p) in self.w.each_mut()
                .zip(p)
            {
                w.resize(p, 0.0);
            }

            self.prime_curve = prime_curve;
            self.length = length;
        }
    }

    pub fn next(&mut self, rate: f64, x: f64) -> f64
    {
        #[allow(invalid_value)]
        let mut z: [_; M] = unsafe {MaybeUninit::uninit().assume_init()};

        for ((z, w), [f_f, f_c]) in z.iter_mut()
            .zip(self.w.iter())
            .zip(self.f.iter_mut())
        {
            *z = w.last()
                .copied()
                .unwrap_or(0.0);

            *z = if *f_f.param.omega <= *f_c.param.omega
            {
                let [_, z] = f_f.filter(rate, *z);
                let [z, _] = f_c.filter(rate, z);
                z
            }
            else
            {
                let [z, z_h] = f_f.filter(rate, *z);
                let [z_l, _] = f_c.filter(rate, z);
                z_h + z_l
            };
        }
        
        let mut y = 0.0;

        z.as_collumn_mut()
            .rmul_matrix_assign(&self.q);

        for ((z, g), w) in z.iter_mut()
            .zip(self.g)
            .zip(self.w.iter_mut())
        {
            z.mul_add_assign(g, x);
            w.shift_right(z);
            y += *z;
        }

        y
    }

    pub fn suspend(&mut self)
    {
        for w in self.w.each_mut()
        {
            w.as_mut_slice()
                .fill(0.0)
        }
    }
}