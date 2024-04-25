extern crate hound;

use crate::lfo::LFO;

mod lfo;
mod envelope;
mod ring_buffer;

fn main() {
    let mut reader = hound::WavReader::open("input_instrument.wav").expect("Failed to open WAV file");
    let spec = reader.spec();
    let samples: Vec<f32> = reader.samples::<i16>()
        .map(|s| s.unwrap() as f32 / i16::MAX as f32)
        .collect();

    let sample_rate = 44100.0;
    let num_taps = 101;        // Number of filter taps (should be odd for symmetry)
    let base_f_low = 100.0; // Base lower cutoff frequency in Hz
    let base_f_high = 3000.0; // Base upper cutoff frequency in Hz
    let mut lfo = LFO::new(4.0, sample_rate as usize);
    let block_size = 64; // Process audio in blocks to allow for real-time parameter changes

    let mut processed_samples = Vec::new();
    let mut previous_samples: Vec<f32> = Vec::new();

    for start in (0..samples.len()).step_by(block_size) {
        let end = std::cmp::min(start + block_size, samples.len());
        let block_samples = &samples[start..end];

        let mut lfo_values = vec![0.0; block_size];
        lfo.get_block(&mut lfo_values);
        // Modulate filter parameters based on LFO
        let mod_f_low = base_f_low + (lfo_values[0] * (base_f_high - base_f_low));
        let mod_f_high = base_f_high + (lfo_values[0] * (base_f_high - base_f_low));
        let taps = bandpass_fir(num_taps, mod_f_low as f64, mod_f_high as f64, sample_rate);


        let filtered_block = apply_fir_filter_blockwise(block_samples, &taps, &mut previous_samples);
        processed_samples.extend_from_slice(&filtered_block);
    }

    let mut writer = hound::WavWriter::create("outputfir.wav", spec).expect("Failed to create WAV file");
    for sample in processed_samples {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer.write_sample(amplitude).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize WAV file");
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