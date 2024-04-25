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
