use statrs::distribution::ContinuousCDF;
use crate::distribution::SignedRank;
use crate::statistics::*;

/// Implements the [Wilcoxon signed rank test](https://en.wikipedia.org/wiki/Wilcoxon_signed-rank_test).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WilcoxonWTest {
    estimate: (f64, f64),
    effect_size: f64,
    p_value: f64
}

impl WilcoxonWTest {
    /// Run Wilcoxon signed rank test on samples `x` and `y`.
    pub fn paired (x: &Vec<f64>, y: &Vec<f64>) -> statrs::Result<WilcoxonWTest> {
        let d: Vec<_> = x.iter().zip(y).map(|(x, y)| (x - y).abs()).collect();
        let (ranks, _) = (&d).ranks();
        let mut w = (0.0, 0.0);
        let mut non_zero = 0;

        for ((x, y), rank) in x.iter().zip(y).zip(ranks) {
            if x < y {
                non_zero += 1;
                w.0 += rank
            } else if x > y {
                non_zero += 1;
                w.1 += rank
            }
        }

        let small_w = if w.0 < w.1 { w.0 } else { w.1 };
        let distribution = SignedRank::new(d.len(), non_zero)?;
        let p_value = distribution.cdf(small_w);

        let n = (&d).n();
        let rank_sum = n * (n + 1.0) / 2.0;
        let effect_size = small_w / rank_sum;

        Ok(WilcoxonWTest {
            effect_size,
            estimate: w,
            p_value: p_value
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn paired() {
        let x = vec!(8.0, 6.0, 5.5, 11.0, 8.5, 5.0, 6.0, 6.0);
        let y = vec!(8.5, 9.0, 6.5, 10.5, 9.0, 7.0, 6.5, 7.0);
        let test = super::WilcoxonWTest::paired(&x, &y).unwrap();
        assert_eq!(test.estimate, (33.5, 2.5));
        // assert_eq!(test.p_value, 0.02779);
    }

    #[test]
    fn paired_2() {
        let x = vec!(209.0, 200.0, 177.0, 169.0, 159.0, 169.0, 187.0, 198.0);
        let y = vec!(151.0, 168.0, 147.0, 164.0, 166.0, 163.0, 176.0, 188.0);
        let test = super::WilcoxonWTest::paired(&x, &y).unwrap();
        assert_eq!(test.estimate, (3.0, 33.0));
        // assert_eq!(test.p_value, 0.0390625);
    }
}
