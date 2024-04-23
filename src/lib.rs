use std::sync::Arc;

use nih_plug::prelude::*;
use sdr::FIR;

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
    envelope: Envelope,
    lfo: LFO,
    previous_samples_list: Vec<Vec<f32>>,  // Buffer for storing the last N-1 samples between process calls
    sample_rate: f64
}

#[derive(Params)]
struct WahwahParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "attack_rate"]
    pub attack_rate: FloatParam,
    #[id = "decay_rate"]
    pub decay_rate: FloatParam,
    #[id = "onset_threshold"]
    pub onset_threshold: FloatParam,
    #[id = "reset_threshold"]
    pub reset_threshold: FloatParam,
    #[id = "use_onset_detection"]
    pub use_onset_detection: BoolParam,
    #[id = "lfo_freq"]
    pub lfo_freq: FloatParam,
    #[id = "base_low_filter"]
    pub base_low_filter: FloatParam,
    #[id = "base_high_filter"]
    pub base_high_filter: FloatParam,

}

impl Default for Wahwah {
    fn default() -> Self {
        Self {
            params: Arc::new(WahwahParams::default()),
            envelope: Envelope::new(0.001, 0.0001, 0.0, 0.05),
            lfo: LFO::new(4.0, 44100),
            previous_samples_list: Vec::new(),  // Initially empty
            sample_rate: 44100.0,
        }
    }
}

impl Default for WahwahParams {
    fn default() -> Self {
        Self {
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
            attack_rate: FloatParam::new(
                "Envelope Attack Rate",
                0.001,
                FloatRange::Linear { min: (0.001), max: (0.1) },
            ),
            decay_rate: FloatParam::new(
                "Envelope Decay Rate",
                0.0001,
                FloatRange::Linear { min: (0.5), max: (10.0) },
            ),
            onset_threshold: FloatParam::new(
                "Onset Threshold",
                0.15,
                FloatRange::Linear {
                    min: (0.0),
                    max: (1.0),
                },
            ),
            reset_threshold: FloatParam::new(
                "Reset Threshold",
                0.05,
                FloatRange::Linear {
                    min: (0.0),
                    max: (1.0),
                },
            ),
            use_onset_detection: BoolParam::new(
                "Use Onset Detection",
                false,
            ),
            lfo_freq: FloatParam::new(
                "LFO Frequency",
                4.0,
                FloatRange::Linear {
                    min: (0.0),
                    max: (20.0),
                },
            ),
            base_low_filter: FloatParam::new(
                "Bandpass Low Frequency",
                100.0,
                FloatRange::Linear {
                    min: (0.0),
                    max: (20000.0),
                },
            ),
            base_high_filter: FloatParam::new(
                "Bandpass High Frequency",
                3000.0,
                FloatRange::Linear {
                    min: (0.0),
                    max: (20000.0),
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
        let num_channels = _audio_io_layout.main_input_channels;
        self.sample_rate = _buffer_config.sample_rate as f64;
        let num_taps = 101;  // n   ber of taps in your FIR filter
        for _ in 0..num_channels.unwrap().into(){
            let mut new_vec = Vec::new();
            new_vec.resize(num_taps - 1, 0.0);
            self.previous_samples_list.push(new_vec);
        }
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
        let gain = self.params.gain.smoothed.next();
        let attack_rate = self.params.attack_rate.smoothed.next();
        let decay_rate = self.params.decay_rate.smoothed.next();
        let onset_threshold = self.params.onset_threshold.smoothed.next();
        let reset_threshold = self.params.reset_threshold.smoothed.next();
        let use_onset_detection = self.params.use_onset_detection.value();

        let lfo_freq = self.params.lfo_freq.smoothed.next();
        let base_f_low = self.params.base_low_filter.smoothed.next();
        let base_f_high = self.params.base_high_filter.smoothed.next();

        self.envelope = Envelope::new(attack_rate, decay_rate, onset_threshold, reset_threshold);
        self.lfo = LFO::new(lfo_freq, self.sample_rate as usize);

        let num_taps = 101;
        let sample_rate = self.sample_rate;
        let block_samples = buffer.as_slice();
        let mut lfo_values = vec![0.0; block_samples.len()];
        self.lfo.get_block(&mut lfo_values);

        let mut channel_index = 0;
        for channel_samples in block_samples{
            let mod_f_low = base_f_low + (lfo_values[channel_index] * (base_f_high - base_f_low));
            let mod_f_high = base_f_high + (lfo_values[channel_index] * (base_f_high - base_f_low));
            let taps = bandpass_fir(num_taps, mod_f_low as f64, mod_f_high as f64, sample_rate);
            let filtered_block = apply_fir_filter_blockwise(&channel_samples, &taps, &mut self.previous_samples_list[channel_index]);

            for (sample, &processed) in channel_samples.iter_mut().zip(filtered_block.iter()) {
                let env_value = self.envelope.process_one_sample(sample);
                let orig_sample = *sample;

                if use_onset_detection{
                    let g = gain * env_value;
                    *sample = processed * g + orig_sample*(1.0-g);
                }else {
                    *sample = processed * gain;
                }
            }
            channel_index += 1;
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

    // Update previous_samples for the next block
    if input.len() >= num_taps {
        previous_samples.clear();
        previous_samples.extend_from_slice(&input[input.len() - num_taps + 1..]);
    } else {
        // Maintain sliding window of samples
        previous_samples.drain(0..input.len());
        previous_samples.extend_from_slice(input);
    }

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
