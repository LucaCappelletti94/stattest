use core::fmt::Debug;
use std::ops::Sub;

use crate::distribution::SignedRank;
use crate::statistics::*;
use crate::traits::abs::Abs;
use crate::traits::zero::Zero;
use statrs::distribution::ContinuousCDF;

use super::StatisticalTest;
use core::cmp::Ordering;

#[cfg(feature = "voracious_radix_sort")]
use voracious_radix_sort::{RadixSort, RadixKey, Radixable};

/// A struct wrapper for sorting values of type T by absolute value.
#[derive(Copy, Clone, Debug)]

#[repr(transparent)]
pub struct AbsWrapper<T> {
    value: T,
}

impl<T> From<T> for AbsWrapper<T> {
    fn from(value: T) -> Self {
        AbsWrapper { value }
    }
}

impl<T: PartialOrd + Copy + Abs<Output = T>> PartialOrd for AbsWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.abs().partial_cmp(&other.value.abs())
    }
}

impl<T: PartialEq + Copy + Abs<Output = T>> PartialEq for AbsWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value.abs() == other.value.abs()
    }
}

#[cfg(feature = "voracious_radix_sort")]
impl Radixable<f32> for AbsWrapper<f32> {
    type Key = f32;

    #[inline]
    fn key(&self) -> Self::Key {
        self.value.abs()
    }
}

#[cfg(feature = "voracious_radix_sort")]
impl Radixable<f64> for AbsWrapper<f64> {
    type Key = f64;

    #[inline]
    fn key(&self) -> Self::Key {
        self.value.abs()
    }
}

#[cfg(feature = "voracious_radix_sort")]
impl Radixable<i8> for AbsWrapper<i8> {
    type Key = i8;

    #[inline]
    fn key(&self) -> Self::Key {
        self.value.abs()
    }
}

#[cfg(feature = "voracious_radix_sort")]
impl Radixable<i16> for AbsWrapper<i16> {
    type Key = i16;

    #[inline]
    fn key(&self) -> Self::Key {
        self.value.abs()
    }
}

#[cfg(feature = "voracious_radix_sort")]
impl Radixable<i32> for AbsWrapper<i32> {
    type Key = i32;

    #[inline]
    fn key(&self) -> Self::Key {
        self.value.abs()
    }
}

#[cfg(feature = "voracious_radix_sort")]
impl Radixable<i64> for AbsWrapper<i64> {
    type Key = i64;

    #[inline]
    fn key(&self) -> Self::Key {
        self.value.abs()
    }
}

/// Implements the [Wilcoxon signed rank test](https://en.wikipedia.org/wiki/Wilcoxon_signed-rank_test).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WilcoxonWTest {
    estimate: (f64, f64),
    effect_size: f64,
    p_value: f64,
}

impl WilcoxonWTest {
    pub fn paired<I, J>(x: I, y: J) -> statrs::Result<WilcoxonWTest>
    where
        I: IntoIterator,
        J: IntoIterator<Item = I::Item>,
        I::IntoIter: ExactSizeIterator,
        J::IntoIter: ExactSizeIterator,
        I::Item: Copy + Debug + Sub<I::Item>,
        <I::Item as Sub<I::Item>>::Output:
            Abs<Output = <I::Item as Sub<I::Item>>::Output> + PartialOrd + Zero + Copy + Debug,
    {
        WilcoxonWTest::paired_with_sort(x, y, |x| {
            x.sort_unstable_by(|a, b| {
                a.abs()
                    .partial_cmp(&b.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        })
    }

    #[cfg(feature = "voracious_radix_sort")]
    pub fn voracious_paired<I, J>(x: I, y: J) -> statrs::Result<WilcoxonWTest>
    where
        I: IntoIterator,
        J: IntoIterator<Item = I::Item>,
        I::IntoIter: ExactSizeIterator,
        J::IntoIter: ExactSizeIterator,
        I::Item: Copy + Debug + Sub<I::Item>,
        <I::Item as Sub<I::Item>>::Output: Abs<Output = <I::Item as Sub<I::Item>>::Output>
            + PartialOrd
            + Zero
            + Copy
            + Debug
            + RadixKey
            + Abs,
        AbsWrapper<<I::Item as Sub<I::Item>>::Output>: Radixable<<I::Item as Sub<I::Item>>::Output>,
    {
        WilcoxonWTest::paired_with_sort(x, y, |x: &mut [<I::Item as Sub<I::Item>>::Output]| {
            // Since the AbsWrapper is a transparent wrapper, we can just cast the slice to a slice of AbsWrapper
            let x: &mut [AbsWrapper<<I::Item as Sub<I::Item>>::Output>] =
                unsafe { std::mem::transmute(x) };
            x.voracious_sort();
        })
    }

    /// Run Wilcoxon signed rank test on samples `x` and `y`.
    pub fn paired_with_sort<I, J, F>(x: I, y: J, sort: F) -> statrs::Result<WilcoxonWTest>
    where
        I: IntoIterator,
        J: IntoIterator<Item = I::Item>,
        I::IntoIter: ExactSizeIterator,
        J::IntoIter: ExactSizeIterator,
        I::Item: Copy + Debug + Sub<I::Item>,
        <I::Item as Sub<I::Item>>::Output:
            Abs<Output = <I::Item as Sub<I::Item>>::Output> + PartialOrd + Zero + Copy + Debug,
        F: Fn(&mut [<I::Item as Sub<I::Item>>::Output]),
    {
        let x_iter = x.into_iter();
        let y_iter = y.into_iter();
        let x_len: usize = x_iter.len();
        let y_len = y_iter.len();

        assert_eq!(x_len, y_len, "Samples must have the same length");

        let mut deltas: Vec<<I::Item as Sub<I::Item>>::Output> =
            x_iter.zip(y_iter).map(|(x, y)| x - y).collect();

        sort(&mut deltas);

        let mut tie_solver = ResolveTies::new(
            deltas.iter().copied(),
            <<I::Item as Sub<I::Item>>::Output as Abs>::abs,
        );

        let mut estimate = (0.0, 0.0);
        let mut zeroes = 0;

        for (rank, delta) in &mut tie_solver {
            if delta < <<I::Item as Sub<I::Item>>::Output as Zero>::ZERO {
                estimate.0 += rank;
            } else if delta > <<I::Item as Sub<I::Item>>::Output as Zero>::ZERO {
                estimate.1 += rank;
            } else {
                zeroes += 1;
            }
        }

        let estimate_small = if estimate.0 < estimate.1 {
            estimate.0
        } else {
            estimate.1
        };
        let distribution = SignedRank::new(x_len, zeroes, tie_solver.tie_correction())?;
        let p_value = distribution.cdf(estimate_small);

        let n = x_len as f64;
        let rank_sum = n * (n + 1.0) / 2.0;
        let effect_size = estimate_small / rank_sum;

        Ok(WilcoxonWTest {
            effect_size,
            estimate,
            p_value,
        })
    }
}

impl StatisticalTest for WilcoxonWTest {
    type Estimate = (f64, f64);

    fn estimate(&self) -> (f64, f64) {
        self.estimate
    }

    fn p_value(&self) -> f64 {
        self.p_value
    }

    fn effect_size(&self) -> f64 {
        self.effect_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_float {
        ($($float:ty),*) => {
            $(
                paste::paste!{
                    #[test]
                    fn [<paired_ $float>]() {
                        let x: Vec<$float> = vec![8.0, 6.0, 5.5, 11.0, 8.5, 5.0, 6.0, 6.0];
                        let y: Vec<$float> = vec![8.5, 9.0, 6.5, 10.5, 9.0, 7.0, 6.5, 7.0];
                        let test = WilcoxonWTest::paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (33.5, 2.5));
                        assert_eq!(test.p_value(), 0.027785782704095215);
                        assert_eq!(test.effect_size(), 0.06944444444444445);
                    }

                    #[test]
                    #[cfg(feature="voracious_radix_sort")]
                    fn [<voracious_paired_ $float>]() {
                        let x: Vec<$float> = vec![8.0, 6.0, 5.5, 11.0, 8.5, 5.0, 6.0, 6.0];
                        let y: Vec<$float> = vec![8.5, 9.0, 6.5, 10.5, 9.0, 7.0, 6.5, 7.0];
                        let test = WilcoxonWTest::voracious_paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (33.5, 2.5));
                        assert_eq!(test.p_value(), 0.027785782704095215);
                        assert_eq!(test.effect_size(), 0.06944444444444445);
                    }

                    #[test]
                    fn [<paired2_ $float>]() {
                        let x: Vec<$float> = vec![209.0, 200.0, 177.0, 169.0, 159.0, 169.0, 187.0, 198.0];
                        let y: Vec<$float> = vec![151.0, 168.0, 147.0, 164.0, 166.0, 163.0, 176.0, 188.0];
                        let test = WilcoxonWTest::paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (3.0, 33.0));
                        assert_eq!(test.p_value(), 0.0390625);
                        assert_eq!(test.effect_size(), 0.08333333333333333);
                    }

                    #[test]
                    #[cfg(feature="voracious_radix_sort")]
                    fn [<voracious_paired2_ $float>]() {
                        let x: Vec<$float> = vec![209.0, 200.0, 177.0, 169.0, 159.0, 169.0, 187.0, 198.0];
                        let y: Vec<$float> = vec![151.0, 168.0, 147.0, 164.0, 166.0, 163.0, 176.0, 188.0];
                        let test = WilcoxonWTest::voracious_paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (3.0, 33.0));
                        assert_eq!(test.p_value(), 0.0390625);
                        assert_eq!(test.effect_size(), 0.08333333333333333);
                    }
                }
            )*
        }
    }

    test_float!(f32, f64);

    macro_rules! test_signed_integer {
        ($($integer:ty),*) => {
            $(
                paste::paste!{
                    #[test]
                    fn [<paired_ $integer>]() {
                        let x: Vec<$integer> = vec![16, 12, 11, 22, 17, 10, 12, 12];
                        let y: Vec<$integer> = vec![17, 18, 13, 21, 18, 14, 13, 14];
                        let test = WilcoxonWTest::paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (33.5, 2.5));
                        assert_eq!(test.p_value(), 0.027785782704095215);
                        assert_eq!(test.effect_size(), 0.06944444444444445);
                    }

                    #[test]
                    #[cfg(feature="voracious_radix_sort")]
                    fn [<voracious_paired_ $integer>]() {
                        let x: Vec<$integer> = vec![16, 12, 11, 22, 17, 10, 12, 12];
                        let y: Vec<$integer> = vec![17, 18, 13, 21, 18, 14, 13, 14];
                        let test = WilcoxonWTest::voracious_paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (33.5, 2.5));
                        assert_eq!(test.p_value(), 0.027785782704095215);
                        assert_eq!(test.effect_size(), 0.06944444444444445);
                    }

                    #[test]
                    fn [<paired2_ $integer>]() {
                        // This test is different from the analogous floating point
                        // one because we needed to fit the values inside a i8
                        let x: Vec<$integer> = vec![109, 100, 77, 69, 59, 69, 87, 98];
                        let y: Vec<$integer> = vec![121, 68, 47, 64, 66, 63, 76, 88];
                        let test = WilcoxonWTest::paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (9.0, 27.0));
                        assert_eq!(test.p_value(), 0.25);
                        assert_eq!(test.effect_size(), 0.25);
                    }

                    #[test]
                    #[cfg(feature="voracious_radix_sort")]
                    fn [<voracious_paired2_ $integer>]() {
                        // This test is different from the analogous floating point
                        // one because we needed to fit the values inside a i8
                        let x: Vec<$integer> = vec![109, 100, 77, 69, 59, 69, 87, 98];
                        let y: Vec<$integer> = vec![121, 68, 47, 64, 66, 63, 76, 88];
                        let test = WilcoxonWTest::voracious_paired(&x, &y).unwrap();
                        assert_eq!(test.estimate(), (9.0, 27.0));
                        assert_eq!(test.p_value(), 0.25);
                        assert_eq!(test.effect_size(), 0.25);
                    }
                }
            )*
        }
    }

    test_signed_integer!(i8, i16, i32, i64);
}
