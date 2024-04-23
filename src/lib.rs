mod vibrato;
mod ring_buffer;
mod lfo;
mod envelope;

use envelope::Envelope;
use nih_plug::prelude::*;
use std::sync::Arc;
use synthrs::filter::{bandpass_filter, cutoff_from_frequency};
use vibrato::Vibrato;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Wahwah {
    params: Arc<WahwahParams>,
    vibrato: Vibrato,
    envelope: Envelope,
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
            envelope: Envelope::new(0.001, 0.0001, 0.3, 0.05)
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
                }
            ),
            onset_threshold: FloatParam::new(
                "Onset Threshold",
                0.0,
                FloatRange::Linear {
                    min: (0.0),
                    max: (1.0),
                }
            )
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
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let onset_threshold = self.params.onset_threshold.smoothed.next();
            let gain = self.params.gain.smoothed.next();
            let delay = self.params.delay.smoothed.next();
            let freq = self.params.freq.smoothed.next();
            self.vibrato.set_delay(delay);
            self.envelope.set_threshold(onset_threshold);
            // self.vibrato.set_freq(freq);

            for mut sample in channel_samples {
                let env_value = self.envelope.process_one_sample(sample);
                // let env_value = 0.5;
                let orig_sample = *sample;
                self.vibrato.process_one_sample(&mut sample);

                let g = gain * env_value;
                *sample = *sample * g + orig_sample*(1.0-g);
            }
        }

        ProcessStatus::Normal
    }
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
