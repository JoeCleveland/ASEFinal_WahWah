use std::sync::Arc;

use nih_plug::prelude::*;
use sdr::FIR;
use synthrs::filter::{bandpass_filter, cutoff_from_frequency};

use envelope::Envelope;
use vibrato::Vibrato;

use crate::lfo::LFO;

mod vibrato;
mod ring_buffer;
mod lfo;
mod envelope;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Wahwah {
    params: Arc<WahwahParams>,
    vibrato: Vibrato,
    envelope: Envelope,
    lfo: LFO,
    // fir: FIR<f32>
}

#[derive(Params)]
struct WahwahParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "delay"]
    pub delay: FloatParam,
    #[id = "freq"]
    pub freq: FloatParam,
    #[id = "onset_threshold"]
    pub onset_threshold: FloatParam,
}

impl Default for Wahwah {
    fn default() -> Self {
        Self {
            params: Arc::new(WahwahParams::default()),
            vibrato: Vibrato::new(4.0, 1.0, 44100),
            envelope: Envelope::new(0.001, 0.0001, 0.3, 0.05),
            lfo: LFO::new(4.0, 44100),
        }
    }
}

impl Default for WahwahParams {
    fn default() -> Self {
        Self {
            // todo: add threshold for onset detection
            delay: FloatParam::new(
                "Delay",
                0.0,
                FloatRange::Linear { min: (0.001), max: (0.1) },
            ),
            freq: FloatParam::new(
                "Freq",
                0.0,
                FloatRange::Linear { min: (0.5), max: (10.0) },
            ),
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Linear {
                    min: (0.0),
                    max: (1.0),
                },
            ),
            onset_threshold: FloatParam::new(
                "Onset Threshold",
                0.0,
                FloatRange::Linear {
                    min: (0.0),
                    max: (1.0),
                },
            ),
        }
    }
}

impl Plugin for Wahwah {
    const NAME: &'static str = "Wahwah";
    const VENDOR: &'static str = "JCleveland";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "jcleveland35@gatech.edu";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        self.vibrato.set_delay(0.1);
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let num_taps = 101;
        let sample_rate = 44100.0;
        let block_size = 64;
        let mut previous_samples = Vec::new();

        for mut channel_samples in buffer.iter_samples() {
            let mut processed_samples = Vec::new();
            let mut current_block = Vec::with_capacity(block_size);
            let mut i = 0;
            let mut lfo_values = vec![0.0; block_size];
            self.lfo.get_block(&mut lfo_values);

            for sample in channel_samples.iter_mut() {
                current_block.push(*sample);
                i += 1;
                if current_block.len() == block_size || i == channel_samples.len() - 1 {
                    let mod_f_low = 100.0 + (lfo_values[0] * 2900.0);
                    let mod_f_high = 3000.0 + (lfo_values[0] * 2900.0);
                    let taps = bandpass_fir(num_taps, mod_f_low as f64, mod_f_high as f64, sample_rate);
                    let filtered_block = apply_fir_filter_blockwise(&current_block, &taps, &mut previous_samples);
                    processed_samples.extend(filtered_block);
                    current_block.clear();
                }
            }

            // Replace channel samples with processed samples
            for (sample, &processed) in channel_samples.iter_mut().zip(processed_samples.iter()) {
                *sample = processed;
            }
        }

        ProcessStatus::Normal
    }
}

fn bandpass_fir(num_taps: usize, f_low: f64, f_high: f64, sample_rate: f64) -> Vec<f64> {
    let mut taps = vec![0.0; num_taps];
    let center = num_taps / 2;
    let fl = f_low / sample_rate;
    let fh = f_high / sample_rate;
    for i in 0..num_taps {
        let n = i as f64 - center as f64;

        // Avoid division by zero in the sinc function calculation
        if n == 0.0 {
            taps[i] = 2.0 * (fh - fl);
        } else {
            taps[i] = (2.0 * fh * (f64::sin(2.0 * std::f64::consts::PI * fh * n) / (2.0 * std::f64::consts::PI * fh * n))) -
                (2.0 * fl * (f64::sin(2.0 * std::f64::consts::PI * fl * n) / (2.0 * std::f64::consts::PI * fl * n)));
        }

        // Apply a Hamming window to the sinc function
        taps[i] *= 0.54 - 0.46 * f64::cos(2.0 * std::f64::consts::PI * i as f64 / (num_taps - 1) as f64);
    }

    taps
}

fn apply_fir_filter_blockwise(input: &[f32], taps: &Vec<f64>, previous_samples: &mut Vec<f32>) -> Vec<f32> {
    let num_taps = taps.len();
    let num_samples = input.len();
    let mut output = vec![0.0; num_samples];

    // Ensure the buffer has enough samples to cover the FIR filter requirement
    previous_samples.resize(num_taps - 1, 0.0);

    // Combine previous and current samples
    let combined_samples = previous_samples.iter().cloned().chain(input.iter().cloned()).collect::<Vec<f32>>();

    for i in 0..num_samples {
        let mut acc = 0.0;
        for j in 0..num_taps {
            if i + num_taps <= combined_samples.len() {
                acc += combined_samples[i + j] * taps[j] as f32;
            }
        }
        output[i] = acc;
    }

    // Update the previous_samples buffer for the next block
    *previous_samples = combined_samples[num_samples..].to_vec();

    output
}

impl ClapPlugin for Wahwah {
    const CLAP_ID: &'static str = "com.your-domain.WahWah";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Wahwah {
    const VST3_CLASS_ID: [u8; 16] = *b"ase_final_wahwah";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Wahwah);
nih_export_vst3!(Wahwah);
