use crate::Byte;
use crate::audio::registers::nrx2::NRX2;

const MAX_VOLUME: Byte = 15;

pub struct Envelope {
    volume: Byte,
    timer: Byte,
}

impl Envelope {
    pub fn new(volume: Byte) -> Self {
        Self { volume, timer: 0 }
    }

    pub fn volume(&self) -> Byte {
        self.volume
    }

    /// Normalized volume, from 0.0 to 1.0
    pub fn output_volume(&self) -> f32 {
        self.volume as f32 / MAX_VOLUME as f32
    }

    pub fn trigger(&mut self, nrx2: &NRX2) {
        self.volume = nrx2.initial_volume();
        self.reload_timer(nrx2);
    }

    pub fn clear(&mut self) {
        self.volume = 0;
        self.timer = 0;
    }

    pub fn tick(&mut self, nrx2: &NRX2) {
        if nrx2.pace() == 0 {
            return;
        }

        self.timer = self.timer.saturating_sub(1);

        if self.timer != 0 {
            return;
        }

        self.reload_timer(nrx2);

        if nrx2.is_increasing() {
            if self.volume < MAX_VOLUME {
                self.volume += 1;
            }
        } else if self.volume > 0 {
            self.volume -= 1;
        }
    }

    /// A pace of 0 is treated as 8 when reloading the timer
    fn reload_timer(&mut self, nrx2: &NRX2) {
        self.timer = match nrx2.pace() {
            0 => 8,
            pace => pace,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::registers::AudioRegister;

    fn nrx2(value: Byte) -> NRX2 {
        let mut nrx2 = NRX2::new_nr22();
        nrx2.write(value);
        nrx2
    }

    #[test]
    fn it_loads_the_initial_volume_on_trigger() {
        let mut fixture = Envelope::new(0);
        fixture.trigger(&nrx2(0xA0));

        assert_eq!(fixture.volume(), 0xA);
    }

    #[test]
    fn it_decreases_the_volume_every_pace_ticks() {
        let nrx2 = nrx2(0x82);
        let mut fixture = Envelope::new(0);
        fixture.trigger(&nrx2);

        fixture.tick(&nrx2);
        assert_eq!(fixture.volume(), 8);

        fixture.tick(&nrx2);
        assert_eq!(fixture.volume(), 7);

        fixture.tick(&nrx2);
        assert_eq!(fixture.volume(), 7);

        fixture.tick(&nrx2);
        assert_eq!(fixture.volume(), 6);
    }

    #[test]
    fn it_increases_the_volume_up_to_the_maximum() {
        let nrx2 = nrx2(0xE9);
        let mut fixture = Envelope::new(0);
        fixture.trigger(&nrx2);

        for _ in 0..10 {
            fixture.tick(&nrx2);
        }

        assert_eq!(fixture.volume(), 15);
    }

    #[test]
    fn it_does_not_go_below_zero() {
        let nrx2 = nrx2(0x11);
        let mut fixture = Envelope::new(0);
        fixture.trigger(&nrx2);

        fixture.tick(&nrx2);
        fixture.tick(&nrx2);

        assert_eq!(fixture.volume(), 0);
    }

    #[test]
    fn it_keeps_the_volume_when_pace_is_zero() {
        let nrx2 = nrx2(0x80);
        let mut fixture = Envelope::new(0);
        fixture.trigger(&nrx2);

        for _ in 0..16 {
            fixture.tick(&nrx2);
        }

        assert_eq!(fixture.volume(), 8);
    }

    #[test]
    fn it_resets_the_volume_on_clear() {
        let mut fixture = Envelope::new(15);
        fixture.clear();

        assert_eq!(fixture.volume(), 0);
    }
}
