use std::sync::atomic::{AtomicU16, Ordering};

use vst::prelude::PluginParameters;
use vst::util::AtomicFloat;

use crate::*;

const EQ_MAX: f32 = 10.0;
pub const REVERB_CURVE: f32 = 0.01304303747559888;

pub const FREQUENCY_MIN: f32 = 20.0;
pub const FREQUENCY_MAX: f32 = 20000.0;

const PRIMES_MIN: f32 = 0.1;
const PRIMES_MAX: f32 = 10.0;

pub enum ReverbParam
{
    Gain,
    Wet,
    Dry,
    Feedback,
    StereoSeparation,
    Floor,
    Ceiling,
    Prescence,
    Mids,
    Mud,
    Primes,
    Length,
    Phase
}

impl ReverbParam
{
    pub const VARIANT_COUNT: usize = core::mem::variant_count::<Self>();

    pub const VARIANTS: [Self; Self::VARIANT_COUNT] = [
        Self::Gain,
        Self::Wet,
        Self::Dry,
        Self::Feedback,
        Self::StereoSeparation,
        Self::Floor,
        Self::Ceiling,
        Self::Prescence,
        Self::Mids,
        Self::Mud,
        Self::Primes,
        Self::Length,
        Self::Phase
    ];
}

pub struct ReverbParameters
{
    pub gain: AtomicFloat,
    pub wet: AtomicFloat,
    pub dry: AtomicFloat,
    pub feedback: AtomicFloat,
    pub stereo_separation: AtomicFloat,
    pub ceiling: AtomicFloat,
    pub floor: AtomicFloat,
    pub prescence: AtomicFloat,
    pub mids: AtomicFloat,
    pub mud: AtomicFloat,
    pub primes: AtomicFloat,
    pub length: AtomicFloat,
    pub phase: AtomicU16,
}

impl ReverbParameters
{
    pub fn store(&self, bank: ReverbBank)
    {
        let ReverbBank {
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
        } = bank;
        self.gain.set(gain as f32);
        self.wet.set(wet as f32);
        self.dry.set(dry as f32);
        self.feedback.set(feedback as f32);
        self.stereo_separation.set(stereo_separation as f32);
        self.ceiling.set(ceiling as f32);
        self.floor.set(floor as f32);
        self.prescence.set(prescence as f32);
        self.mids.set(mids as f32);
        self.mud.set(mud as f32);
        self.primes.set(primes as f32);
        self.length.set(length as f32);
        self.phase.store(phase, Ordering::Relaxed);
    }
    pub fn load(&self) -> ReverbBank
    {
        self.into()
    }
}

impl From<ReverbBank> for ReverbParameters
{
    fn from(bank: ReverbBank) -> Self
    {
        let ReverbBank {
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
        } = bank;
        Self {
            gain: AtomicFloat::new(gain as f32),
            wet: AtomicFloat::new(wet as f32),
            dry: AtomicFloat::new(dry as f32),
            feedback: AtomicFloat::new(feedback as f32),
            stereo_separation: AtomicFloat::new(stereo_separation as f32),
            ceiling: AtomicFloat::new(ceiling as f32),
            floor: AtomicFloat::new(floor as f32),
            prescence: AtomicFloat::new(prescence as f32),
            mids: AtomicFloat::new(mids as f32),
            mud: AtomicFloat::new(mud as f32),
            primes: AtomicFloat::new(primes as f32),
            length: AtomicFloat::new(length as f32),
            phase: AtomicU16::new(phase)
        }
    }
}

impl Default for ReverbParameters
{
    fn default() -> Self
    {
        ReverbBank::default().into()
    }
}

impl PluginParameters for ReverbParameters
{
    fn get_parameter_label(&self, index: i32) -> String
    {
        match ReverbParam::VARIANTS.get(index as usize)
        {
            Some(param) => match param
            {
                ReverbParam::Gain => "%",
                ReverbParam::Wet => "%",
                ReverbParam::Dry => "%",
                ReverbParam::Feedback => "%",
                ReverbParam::StereoSeparation => "%",
                ReverbParam::Floor => "Hz",
                ReverbParam::Ceiling => "Hz",
                ReverbParam::Prescence => "%",
                ReverbParam::Mids => "%",
                ReverbParam::Mud => "%",
                ReverbParam::Primes => "",
                ReverbParam::Length => "%",
                ReverbParam::Phase => ""
            },
            None => ""
        }.to_string()
    }

    fn get_parameter_text(&self, index: i32) -> String
    {
        match ReverbParam::VARIANTS.get(index as usize)
        {
            Some(param) => match param
            {
                ReverbParam::Gain => format!("{:.3}", 100.0*self.gain.get().powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Wet => format!("{:.3}", 100.0*self.wet.get().powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Dry => format!("{:.3}", 100.0*self.dry.get().powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Feedback => format!("{:.3}", 100.0*self.feedback.get().powf(1.0/REVERB_CURVE as f32)),
                ReverbParam::StereoSeparation => format!("{:.3}", 100.0*self.stereo_separation.get().powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Floor => format!("{:.3}", self.floor.get()),
                ReverbParam::Ceiling => format!("{:.3}", self.ceiling.get()),
                ReverbParam::Prescence => format!("{:.3}", 100.0*(self.prescence.get()/EQ_MAX).powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Mids => format!("{:.3}", 100.0*(self.mids.get()/EQ_MAX).powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Mud => format!("{:.3}", 100.0*(self.mud.get()/EQ_MAX).powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Primes => format!("{:.3}", self.primes.get()),
                ReverbParam::Length => format!("{:.3}", 100.0*self.length.get().powf(1.0/LOG_CURVE as f32)),
                ReverbParam::Phase => format!("{}", self.phase.load(Ordering::Relaxed)),
            }, 
            None => "".to_string()
        }
    }

    fn get_parameter_name(&self, index: i32) -> String
    {
        match ReverbParam::VARIANTS.get(index as usize)
        {
            Some(param) => match param
            {
                ReverbParam::Gain => "Gain",
                ReverbParam::Wet => "Wet",
                ReverbParam::Dry => "Dry",
                ReverbParam::Feedback => "Feedback",
                ReverbParam::StereoSeparation => "Stereo Separation",
                ReverbParam::Floor => "Floor",
                ReverbParam::Ceiling => "Ceiling",
                ReverbParam::Prescence => "Prescence",
                ReverbParam::Mids => "Mids",
                ReverbParam::Mud => "Mud",
                ReverbParam::Primes => "Primes",
                ReverbParam::Length => "Length",
                ReverbParam::Phase => "Phase"
            },
            None => ""
        }.to_string()
    }

    /// Get the value of parameter at `index`. Should be value between 0.0 and 1.0.
    fn get_parameter(&self, index: i32) -> f32
    {
        match ReverbParam::VARIANTS.get(index as usize)
        {
            Some(param) => match param
            {
                ReverbParam::Gain => self.gain.get().powf(1.0/LOG_CURVE as f32),
                ReverbParam::Wet => self.wet.get().powf(1.0/LOG_CURVE as f32),
                ReverbParam::Dry => self.dry.get().powf(1.0/LOG_CURVE as f32),
                ReverbParam::Feedback => self.feedback.get().powf(1.0/REVERB_CURVE as f32),
                ReverbParam::StereoSeparation => self.stereo_separation.get().powf(1.0/LOG_CURVE as f32),
                ReverbParam::Floor => (self.floor.get().log2() - FREQUENCY_MIN.log2())/(FREQUENCY_MAX.log2() - FREQUENCY_MIN.log2()),
                ReverbParam::Ceiling => (self.ceiling.get().log2() - FREQUENCY_MIN.log2())/(FREQUENCY_MAX.log2() - FREQUENCY_MIN.log2()),
                ReverbParam::Prescence => (self.prescence.get()/EQ_MAX).powf(1.0/LOG_CURVE as f32),
                ReverbParam::Mids => (self.mids.get()/EQ_MAX).powf(1.0/LOG_CURVE as f32),
                ReverbParam::Mud => (self.mud.get()/EQ_MAX).powf(1.0/LOG_CURVE as f32),
                ReverbParam::Primes => (self.primes.get().log2() - PRIMES_MIN.log2())/(PRIMES_MAX.log2() - PRIMES_MIN.log2()),
                ReverbParam::Length => self.length.get().powf(1.0/LOG_CURVE as f32),
                ReverbParam::Phase => self.phase.load(Ordering::Relaxed) as f32/(M*M - 1) as f32
            },
            None => 0.0
        }
    }
    
    fn set_parameter(&self, index: i32, value: f32)
    {
        match ReverbParam::VARIANTS.get(index as usize)
        {
            Some(param) => match param
            {
                ReverbParam::Gain => self.gain.set(value.powf(LOG_CURVE as f32)),
                ReverbParam::Wet => self.wet.set(value.powf(LOG_CURVE as f32)),
                ReverbParam::Dry => self.dry.set(value.powf(LOG_CURVE as f32)),
                ReverbParam::Feedback => self.feedback.set(value.powf(REVERB_CURVE as f32)),
                ReverbParam::StereoSeparation => self.stereo_separation.set(value.powf(LOG_CURVE as f32)),
                ReverbParam::Floor => self.floor.set((value*(FREQUENCY_MAX.log2() - FREQUENCY_MIN.log2()) + FREQUENCY_MIN.log2()).exp2()),
                ReverbParam::Ceiling => self.ceiling.set((value*(FREQUENCY_MAX.log2() - FREQUENCY_MIN.log2()) + FREQUENCY_MIN.log2()).exp2()),
                ReverbParam::Prescence => self.prescence.set(value.powf(LOG_CURVE as f32)*EQ_MAX),
                ReverbParam::Mids => self.mids.set(value.powf(LOG_CURVE as f32)*EQ_MAX),
                ReverbParam::Mud => self.mud.set(value.powf(LOG_CURVE as f32)*EQ_MAX),
                ReverbParam::Primes => self.primes.set((value*(PRIMES_MAX.log2() - PRIMES_MIN.log2()) + PRIMES_MIN.log2()).exp2()),
                ReverbParam::Length => self.length.set(value.powf(LOG_CURVE as f32)),
                ReverbParam::Phase => self.phase.store((value*(M*M - 1) as f32) as u16, Ordering::Relaxed)
            },
            None => ()
        }
    }

    fn change_preset(&self, _preset: i32) {}

    fn get_preset_num(&self) -> i32
    {
        0
    }

    fn set_preset_name(&self, _name: String) {}

    fn get_preset_name(&self, _preset: i32) -> String
    {
        "".to_string()
    }

    fn can_be_automated(&self, index: i32) -> bool
    {
        (index as usize) < ReverbParam::VARIANT_COUNT
    }

    fn get_preset_data(&self) -> Vec<u8>
    {
        self.get_bank_data()
    }

    fn get_bank_data(&self) -> Vec<u8>
    {
        serde_json::to_vec(&self.load()).expect("Serialization error")
    }

    fn load_preset_data(&self, data: &[u8])
    {
        self.load_bank_data(data);
    }

    fn load_bank_data(&self, data: &[u8])
    {
        self.store(serde_json::from_slice(data).expect("Deserialization error"));
    }
}