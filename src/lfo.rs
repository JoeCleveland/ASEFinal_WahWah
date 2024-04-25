use crate::ring_buffer::RingBuffer;

pub struct LFO {
    buffer: RingBuffer<f32>,
    freq: f32,
    sample_rate: usize,
    index: f32
}

impl LFO {
    pub fn new(freq: f32, sample_rate: usize) -> Self {
        let mut lfo = LFO {
            buffer: RingBuffer::new(sample_rate),
            freq: freq,
            sample_rate: sample_rate,
            index: 0.0,
        };

        for i in 0..sample_rate {
            lfo.buffer.push(f32::sin(i as f32 * freq * 2.0 * std::f32::consts::PI / sample_rate as f32));
        }
        return lfo;
    }

    pub fn get_block(&mut self, output: &mut [f32]) {
        for i in 0..output.len() {
            output[i] = self.buffer.get_frac(self.index);
            self.index += self.freq;

            if self.index > self.sample_rate as f32{
                self.index -= self.sample_rate as f32;
            }
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        if f32::abs(self.freq - freq) > 0.001 {
            self.freq = freq;
            self.buffer.reset();
            for i in 0..self.sample_rate {
                self.buffer.push(f32::sin(i as f32 * 2.0 * self.freq * std::f32::consts::PI / self.sample_rate as f32));
            }
        } 
    }
}

#[test]
fn test_lfo() {
    let mut lfo = LFO::new(1.0, 628);
    let mut output = vec![0f32; 628];
    lfo.get_block(output.as_mut_slice());

    assert!(f32::abs(output[0] - 0.0) < 0.00001);
    assert!(f32::abs(output[157] - 1.0) < 0.00001); // PI / 2
    assert!(f32::abs(output[314] - 0.0) < 0.00001); // PI
    assert!(f32::abs(output[471] + 1.0) < 0.00001); // 3*PI / 2
}