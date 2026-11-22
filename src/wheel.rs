//! Tekerlek delta biriktirici (spec §4): yüksek çözünürlüklü tekerlekte 120'nin katı olmayan deltalar.

pub const WHEEL_DELTA: i32 = 120;

#[derive(Default)]
pub struct Acc(i32);

impl Acc {
    /// Delta ekler, tamamlanan çentik sayısını döner (+ yukarı, − aşağı). Yön değişince kalan atılır.
    pub fn push(&mut self, delta: i32) -> i32 {
        if (self.0 > 0 && delta < 0) || (self.0 < 0 && delta > 0) {
            self.0 = 0;
        }
        self.0 += delta;
        let ticks = self.0 / WHEEL_DELTA;
        self.0 %= WHEEL_DELTA;
        ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_notches() {
        let mut a = Acc::default();
        assert_eq!(a.push(120), 1);
        assert_eq!(a.push(-240), -2);
        assert_eq!(a.push(360), 3);
    }

    #[test]
    fn hi_res_accumulates() {
        let mut a = Acc::default();
        assert_eq!(a.push(40), 0);
        assert_eq!(a.push(40), 0);
        assert_eq!(a.push(40), 1);
        assert_eq!(a.push(100), 0);
        assert_eq!(a.push(50), 1); // 100+50=150 → 1, kalan 30
        assert_eq!(a.push(90), 1);
    }

    #[test]
    fn direction_change_drops_remainder() {
        let mut a = Acc::default();
        assert_eq!(a.push(100), 0);
        assert_eq!(a.push(-40), 0); // 100 atıldı, -40 kaldı
        assert_eq!(a.push(-80), -1);
    }
}
