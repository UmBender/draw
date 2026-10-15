#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn math_metrics_table_is_sorted_and_finite() {
        assert!(TABLE.len() > 500, "{} rows", TABLE.len());
        assert!(TABLE.windows(2).all(|w| w[0].0 < w[1].0), "sorted, unique");
        for (ch, m) in TABLE {
            let values = [m.advance, m.x_min, m.x_max, m.y_min, m.y_max, m.italic];
            assert!(values.iter().all(|v| v.is_finite()), "{ch:?}");
            assert!(m.advance >= 0.0 && m.italic >= 0.0, "{ch:?}");
            assert!(m.x_min <= m.x_max && m.y_min <= m.y_max, "{ch:?}");
            assert!(m.advance < 3.0 && m.y_max < 3.0 && m.y_min > -3.0, "{ch:?}");
        }
    }

    #[test]
    fn math_metrics_get_finds_every_row() {
        for (ch, m) in TABLE {
            assert_eq!(get(*ch), Some(*m));
        }
        assert_eq!(get('日'), None);
        assert_eq!(get('\u{10FFFF}'), None);
        assert_eq!(get('\0'), None);
    }
}
