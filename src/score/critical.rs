use rand::Rng;

const PERCENT_TO_RAW: f32 = 2.5;
const MAX_RAW: u8 = 255;
const CRIT_ROLL_MAX: u16 = 250;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Critical(f32);

impl Critical {
    pub fn raw(self) -> u8 {
        (self.0 * PERCENT_TO_RAW).clamp(0.0, MAX_RAW as f32) as u8
    }

    pub fn percentage(self) -> f32 {
        self.0
    }

    pub fn rolls_crit_with(self, roll: u16) -> bool {
        roll < self.raw() as u16
    }

    pub fn rolls_crit(self, rng: &mut impl Rng) -> bool {
        self.rolls_crit_with(rng.gen_range(0..CRIT_ROLL_MAX))
    }
}

impl std::ops::AddAssign<f32> for Critical {
    fn add_assign(&mut self, percent: f32) {
        self.0 += percent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_zero() {
        let crit = Critical::default();
        assert_eq!(crit.percentage(), 0.0);
        assert_eq!(crit.raw(), 0);
    }

    #[test]
    fn add_percentage() {
        let mut crit = Critical::default();
        crit += 10.0;
        assert_eq!(crit.percentage(), 10.0);
        assert_eq!(crit.raw(), 25);
    }

    #[test]
    fn multiple_adds_stack() {
        let mut crit = Critical::default();
        crit += 5.0;
        crit += 3.0;
        crit += 2.0;
        assert_eq!(crit.percentage(), 10.0);
        assert_eq!(crit.raw(), 25);
    }

    #[test]
    fn hundred_percent() {
        let mut crit = Critical::default();
        crit += 100.0;
        assert_eq!(crit.raw(), 250);
    }

    #[test]
    fn raw_clamps_above_max() {
        let mut crit = Critical::default();
        crit += 200.0;
        assert_eq!(crit.raw(), 255);
    }

    #[test]
    fn raw_clamps_below_zero() {
        let mut crit = Critical::default();
        crit += -10.0;
        assert_eq!(crit.raw(), 0);
    }

    #[test]
    fn matches_cpp_ef_critical_70() {
        // C++ EF_CRITICAL=70: (70/10.0) * 2.5 = 17.5 → raw 17
        // Our way: 70/10.0 = 7.0% → raw = 7.0 * 2.5 = 17.5 → 17
        let mut crit = Critical::default();
        crit += 7.0;
        assert_eq!(crit.raw(), 17);
    }

    #[test]
    fn rolls_crit_zero_never_crits() {
        let crit = Critical::default();
        assert!(!crit.rolls_crit_with(0));
    }

    #[test]
    fn rolls_crit_guaranteed_at_100_percent() {
        let mut crit = Critical::default();
        crit += 100.0;
        assert!(crit.rolls_crit_with(0));
        assert!(crit.rolls_crit_with(249));
        assert!(!crit.rolls_crit_with(250));
    }

    #[test]
    fn rolls_crit_50_percent() {
        let mut crit = Critical::default();
        crit += 50.0;
        assert!(crit.rolls_crit_with(0));
        assert!(crit.rolls_crit_with(124));
        assert!(!crit.rolls_crit_with(125));
        assert!(!crit.rolls_crit_with(249));
    }

    #[test]
    fn rolls_crit_with_rng() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);

        let mut always = Critical::default();
        always += 100.0;
        for _ in 0..100 {
            assert!(always.rolls_crit(&mut rng));
        }

        let never = Critical::default();
        for _ in 0..100 {
            assert!(!never.rolls_crit(&mut rng));
        }
    }
}
