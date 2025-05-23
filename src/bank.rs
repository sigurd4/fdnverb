use core::sync::atomic::Ordering;

use crate::{parameters::{ReverbParameters, FREQUENCY_MAX, FREQUENCY_MIN, REVERB_CURVE}, LOG_MID};

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ReverbBank
{
    #[serde(default = "ReverbBank::default_gain")]
    pub gain: f64,
    #[serde(default = "ReverbBank::default_wet")]
    pub wet: f64,
    #[serde(default = "ReverbBank::default_dry")]
    pub dry: f64,
    #[serde(default = "ReverbBank::default_feedback")]
    pub feedback: f64,
    #[serde(default = "ReverbBank::default_stereo_separation")]
    pub stereo_separation: f64,
    #[serde(default = "ReverbBank::default_ceiling")]
    pub ceiling: f64,
    #[serde(default = "ReverbBank::default_floor")]
    pub floor: f64,
    #[serde(default = "ReverbBank::default_prescence")]
    pub prescence: f64,
    #[serde(default = "ReverbBank::default_mids")]
    pub mids: f64,
    #[serde(default = "ReverbBank::default_mud")]
    pub mud: f64,
    #[serde(default = "ReverbBank::default_primes")]
    pub primes: f64,
    #[serde(default = "ReverbBank::default_length")]
    pub length: f64,
    #[serde(default = "ReverbBank::default_phase")]
    pub phase: u16,
}

impl Default for ReverbBank
{
    fn default() -> Self
    {
        Self {
            gain: Self::default_gain(),
            wet: Self::default_wet(),
            dry: Self::default_dry(),
            feedback: Self::default_feedback(),
            stereo_separation: Self::default_stereo_separation(),
            ceiling: Self::default_ceiling(),
            floor: Self::default_floor(),
            prescence: Self::default_prescence(),
            mids: Self::default_mids(),
            mud: Self::default_mud(),
            primes: Self::default_primes(),
            length: Self::default_length(),
            phase: Self::default_phase()
        }
    }
}

impl From<&ReverbParameters> for ReverbBank
{
    fn from(param: &ReverbParameters) -> Self
    {
        let ReverbParameters {
            gain,
            wet,
            dry,
            feedback,
            stereo_separation,
            ceiling,
            floor,
            prescence,
            mids,
            mud,
            primes,
            length,
            phase
        } = param;
        Self {
            gain: gain.get() as f64,
            wet: wet.get() as f64,
            dry: dry.get() as f64,
            feedback: feedback.get() as f64,
            stereo_separation: stereo_separation.get() as f64,
            ceiling: ceiling.get() as f64,
            floor: floor.get() as f64,
            prescence: prescence.get() as f64,
            mids: mids.get() as f64,
            mud: mud.get() as f64,
            primes: primes.get() as f64,
            length: length.get() as f64,
            phase: phase.load(Ordering::Relaxed)
        }
    }
}

impl ReverbBank
{
    fn default_gain() -> f64
    {
        LOG_MID
    }
    fn default_wet() -> f64
    {
        LOG_MID
    }
    fn default_dry() -> f64
    {
        LOG_MID
    }
    fn default_feedback() -> f64
    {
        0.5f64.powf(REVERB_CURVE as f64)
    }
    fn default_stereo_separation() -> f64
    {
        LOG_MID
    }
    fn default_floor() -> f64
    {
        FREQUENCY_MIN as f64
    }
    fn default_ceiling() -> f64
    {
        FREQUENCY_MAX as f64
    }
    fn default_prescence() -> f64
    {
        0.5
    }
    fn default_mids() -> f64
    {
        0.5
    }
    fn default_mud() -> f64
    {
        0.5
    }
    fn default_primes() -> f64
    {
        1.0
    }
    fn default_length() -> f64
    {
        LOG_MID
    }
    fn default_phase() -> u16
    {
        0
    }
}