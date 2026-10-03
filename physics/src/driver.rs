//! Host-independent input and fixed-step batching. Host owns executing steps.
use crate::{Input, DT};
#[derive(Default)]
pub struct Driver {
    controls: [f64; 5],
    heading: f64,
    pending: bool,
    accumulator: f64,
    pub dropped_seconds: f64,
}
impl Driver {
    pub fn control(&mut self, index: usize, value: f64) -> Result<(), &'static str> {
        if index >= 5 || !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err("invalid control");
        }
        if index == 4 && value > 0.0 && self.controls[4] == 0.0 {
            self.pending = true;
        }
        self.controls[index] = value;
        Ok(())
    }
    pub fn heading(&mut self, value: f64) -> Result<(), &'static str> {
        if !value.is_finite() {
            return Err("invalid heading");
        }
        self.heading = value;
        Ok(())
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn alpha(&self) -> f64 {
        self.accumulator / DT
    }
    /// Returns at most 16 inputs. Caller commits each atomically; reset on failure.
    pub fn advance(&mut self, dt: f64) -> Result<Vec<Input>, &'static str> {
        if !dt.is_finite() || dt < 0.0 || dt > 3600.0 {
            return Err("invalid simulation delta");
        }
        if dt == 0.0 {
            self.accumulator = 0.0;
            self.controls = [0.0; 5];
            self.pending = false;
            return Ok(vec![]);
        }
        let total = self.accumulator + dt;
        let due = (total / DT + 1e-9).floor();
        self.accumulator = (total - due * DT).max(0.0);
        self.dropped_seconds += (due - 16.0).max(0.0) * DT;
        let x = self.controls[3] - self.controls[2];
        let y = self.controls[0] - self.controls[1];
        let len = x.hypot(y).max(1.0);
        let (sin, cos) = self.heading.sin_cos();
        let movement = [(cos * x - sin * y) / len, (sin * x + cos * y) / len];
        let count = (due as usize).min(16);
        let mut inputs = Vec::with_capacity(count);
        for _ in 0..count {
            inputs.push(Input {
                movement,
                jump: self.pending || self.controls[4] > 0.0,
            });
            self.pending = false;
        }
        Ok(inputs)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timing_input_pause_and_backlog() {
        let mut d = Driver::default();
        d.control(0, 1.0).unwrap();
        d.control(3, 1.0).unwrap();
        let mut n = 0;
        for _ in 0..60 {
            let batch = d.advance(1.0 / 60.0).unwrap();
            n += batch.len();
            assert!((batch[0].movement[0].hypot(batch[0].movement[1]) - 1.0).abs() < 1e-10);
        }
        assert_eq!(n, 240);
        d.advance(0.0).unwrap();
        d.control(4, 1.0).unwrap();
        d.control(4, 0.0).unwrap();
        assert!(d.advance(DT / 2.0).unwrap().is_empty());
        assert!(d.advance(DT / 2.0).unwrap()[0].jump);
        assert!(!d.advance(DT).unwrap()[0].jump);
        assert_eq!(d.advance(1.0).unwrap().len(), 16);
        assert!(d.dropped_seconds > 0.9);
        assert!(d.advance(f64::NAN).is_err());
    }
}
