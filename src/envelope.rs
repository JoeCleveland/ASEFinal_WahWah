

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
}