//! Map (cursor_dx, cursor_dy) relative to menu center → which sector index
//! is hovered, given `n` sectors arranged clockwise from 12 o'clock.

pub fn sector_at(dx: f64, dy: f64, n: usize, inner_radius: f64, outer_radius: f64) -> Option<usize> {
    if n == 0 {
        return None;
    }
    let r = (dx * dx + dy * dy).sqrt();
    if r < inner_radius || r > outer_radius {
        return None;
    }
    // atan2 returns (-PI, PI]. Convert to "0 at 12 o'clock, increasing
    // clockwise" by negating dy (screen Y points down).
    let theta_math = (-dy).atan2(dx); // 0 = right, increases CCW
    let theta_cw_from_top =
        (std::f64::consts::FRAC_PI_2 - theta_math).rem_euclid(std::f64::consts::TAU);
    let sector_size = std::f64::consts::TAU / n as f64;
    Some(((theta_cw_from_top / sector_size).floor() as usize) % n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_0_is_top() {
        assert_eq!(sector_at(0.0, -100.0, 4, 20.0, 200.0), Some(0));
    }

    #[test]
    fn sector_1_is_right_for_4() {
        assert_eq!(sector_at(100.0, 0.0, 4, 20.0, 200.0), Some(1));
    }

    #[test]
    fn inner_circle_no_sector() {
        assert_eq!(sector_at(5.0, 5.0, 6, 20.0, 200.0), None);
    }

    #[test]
    fn outer_ring_no_sector() {
        assert_eq!(sector_at(300.0, 0.0, 6, 20.0, 200.0), None);
    }

    #[test]
    fn single_item_always_zero() {
        assert_eq!(sector_at(50.0, 50.0, 1, 20.0, 200.0), Some(0));
    }
}
