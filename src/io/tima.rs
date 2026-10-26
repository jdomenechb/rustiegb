use crate::Byte;

#[derive(Default)]
pub struct Tima {
    pub value: Byte,
    remaining_timer_cycles: u32,
}

impl Tima {
    pub fn reset_cycles(&mut self) {
        self.remaining_timer_cycles = 0;
    }

    pub fn single_step(&mut self, divider: u32) -> bool {
        self.remaining_timer_cycles += 1;

        let to_add = (self.remaining_timer_cycles / divider) as u8;
        self.remaining_timer_cycles %= divider;

        let addition_result = self.value.overflowing_add(to_add);

        self.value = addition_result.0;

        addition_result.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_increases_once_every_divider_single_steps() {
        let mut fixture = Tima::default();

        for _ in 0..15 {
            assert!(!fixture.single_step(16));
        }
        assert_eq!(fixture.value, 0);

        assert!(!fixture.single_step(16));
        assert_eq!(fixture.value, 1);

        for _ in 0..16 {
            fixture.single_step(16);
        }
        assert_eq!(fixture.value, 2);
    }

    #[test]
    fn it_reports_overflow() {
        let mut fixture = Tima {
            value: 0xFF,
            ..Default::default()
        };

        for _ in 0..15 {
            assert!(!fixture.single_step(16));
        }

        assert!(fixture.single_step(16));
        assert_eq!(fixture.value, 0);
    }

    #[test]
    fn it_restarts_the_internal_cycles_when_reset() {
        let mut fixture = Tima::default();

        for _ in 0..15 {
            fixture.single_step(16);
        }

        fixture.reset_cycles();
        fixture.single_step(16);

        assert_eq!(fixture.value, 0);
    }
}
