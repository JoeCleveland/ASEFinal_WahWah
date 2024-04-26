#[derive(Debug, PartialEq, Eq)]
enum EnvelopeState {
    WAITING,
    ATTACK,
    DECAY,
    FINAL,
}
pub struct Envelope {
    state: EnvelopeState,
    curr_value: f32,
    attack_rate: f32,
    decay_rate: f32,
    onset_threshold: f32,
    reset_threshold: f32,
}

/// Attack-decay envelope
/// Sits in the WAITING state until amplitude of the process function's input exceeds the onset_threshold,
/// Then proceeds to run through the ATTACK and DECAY stages of envelope and returns to WAITING
impl Envelope {
    pub fn new(attack_rate: f32, decay_rate: f32, onset_threshold: f32, reset_threshold: f32) -> Self {
        return Envelope {
            state: EnvelopeState::WAITING,
            curr_value: 0.0,
            attack_rate: attack_rate,
            decay_rate: decay_rate,
            onset_threshold: onset_threshold,
            reset_threshold: reset_threshold,
        }
    }

    /// Return next value of envelope given current sample
    pub fn process_one_sample(&mut self, sample: &f32) -> f32 {
        if matches!(self.state, EnvelopeState::WAITING) {
            if *sample > self.onset_threshold {
                self.state = EnvelopeState::ATTACK;
            }
        }
        else if matches!(self.state, EnvelopeState::ATTACK) {
            self.curr_value += self.attack_rate;
            if self.curr_value >= 1.0 {
                self.curr_value = 1.0;
                self.state = EnvelopeState::DECAY;
            }
        }
        else if matches!(self.state, EnvelopeState::DECAY) {
            self.curr_value -= self.decay_rate;
            if self.curr_value <= 0.0 {
                self.curr_value = 0.0;
                self.state = EnvelopeState::WAITING;
            }
        } else if matches!(self.state, EnvelopeState::FINAL) {
            if *sample <= self.reset_threshold {
                self.state = EnvelopeState::WAITING;
            }
        }
        return self.curr_value;
    }

    /// Update all parameters of the envlope, can be called each sample
    pub fn set_params(&mut self, attack_rate:f32, decay_rate: f32, onset_threshold: f32, reset_threshold: f32){
        self.attack_rate = attack_rate;
        self.decay_rate = decay_rate;
        self.onset_threshold = onset_threshold;
        self.reset_threshold = reset_threshold;
    }

}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_envelope() {
        let env = Envelope::new(0.1, 0.05, 0.5, 0.2);
        assert_eq!(env.state, EnvelopeState::WAITING);
        assert_eq!(env.curr_value, 0.0);
        assert_eq!(env.attack_rate, 0.1);
        assert_eq!(env.decay_rate, 0.05);
        assert_eq!(env.onset_threshold, 0.5);
        assert_eq!(env.reset_threshold, 0.2);
    }

    #[test]
    fn test_process_one_sample_attack_to_decay() {
        let mut env = Envelope::new(0.05, 0.05, 0.5, 0.2);
        env.state = EnvelopeState::ATTACK;
        env.curr_value = 0.95;
        let sample = 0.6;  // Still above threshold, but should now switch to decay
        let output = env.process_one_sample(&sample);
        assert_eq!(env.state, EnvelopeState::DECAY);
        assert_eq!(output, 1.0);  // Should cap at 1.0
    }

    #[test]
    fn test_process_one_sample_decay_to_waiting() {
        let mut env = Envelope::new(0.1, 0.1, 0.5, 0.2);
        env.state = EnvelopeState::DECAY;
        env.curr_value = 0.1;
        let sample = 0.6;  // Still above the reset_threshold, should not affect state
        let output = env.process_one_sample(&sample);
        assert_eq!(env.state, EnvelopeState::WAITING);
        assert_eq!(output, 0.0);  // Should decay to zero and return to waiting
    }

    #[test]
    fn test_set_params() {
        let mut env = Envelope::new(0.1, 0.05, 0.5, 0.2);
        env.set_params(0.2, 0.1, 0.6, 0.3);
        assert_eq!(env.attack_rate, 0.2);
        assert_eq!(env.decay_rate, 0.1);
        assert_eq!(env.onset_threshold, 0.6);
        assert_eq!(env.reset_threshold, 0.3);
    }

}
