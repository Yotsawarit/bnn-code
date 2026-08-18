use num_bigint::BigInt;
use num_traits::{One, Zero};
use std::time::Instant;

const C3_24: i64 = 640320i64.pow(3) / 24;
const C396_POW4: i64 = 396i64.pow(4);

struct BsResult {
    p: BigInt,
    q: BigInt,
    t: BigInt,
}

fn chudnovsky_bs(a: i64, b: i64) -> BsResult {
    if b - a == 1 {
        if a == 0 {
            return BsResult {
                p: BigInt::one(),
                q: BigInt::one(),
                t: BigInt::from(13591409),
            };
        }
        let p = BigInt::from(-(6 * a - 5)) * (2 * a - 1) * (6 * a - 1);
        let q = BigInt::from(a).pow(3) * C3_24;
        let t = &p * (BigInt::from(545140134) * a + 13591409);
        BsResult { p, q, t }
    } else {
        let m = (a + b) / 2;
        let left = chudnovsky_bs(a, m);
        let right = chudnovsky_bs(m, b);
        BsResult {
            p: &left.p * &right.p,
            q: &left.q * &right.q,
            t: &left.t * &right.q + &left.p * &right.t,
        }
    }
}

fn ramanujan_bs(a: i64, b: i64) -> BsResult {
    if b - a == 1 {
        if a == 0 {
            return BsResult {
                p: BigInt::one(),
                q: BigInt::one(),
                t: BigInt::from(1103),
            };
        }
        let p = BigInt::from((4 * a - 3) * (4 * a - 2) * (4 * a - 1) * (4 * a));
        let q = BigInt::from(a * a * a * a * C396_POW4);
        let t = &p * BigInt::from(1103 + 26390 * a);
        BsResult { p, q, t }
    } else {
        let m = (a + b) / 2;
        let left = ramanujan_bs(a, m);
        let right = ramanujan_bs(m, b);
        BsResult {
            p: &left.p * &right.p,
            q: &left.q * &right.q,
            t: &left.t * &right.q + &left.p * &right.t,
        }
    }
}

fn integer_sqrt(n: &BigInt) -> BigInt {
    if n.is_zero() || n.is_one() {
        return n.clone();
    }
    let bits = n.bits();
    let mut x = BigInt::from(2u64) << (bits / 2);
    loop {
        let y = (&x + n / &x) >> 1;
        if y >= x {
            return x;
        }
        x = y;
    }
}

fn format_pi(scaled: &BigInt, digits: usize) -> String {
    let s = scaled.to_string();
    let len = s.len();
    if len <= digits {
        let padding = "0".repeat(digits - len + 1);
        format!("0.{}{}", padding, s)
    } else {
        let int_part = &s[..len - digits];
        let frac_part = &s[len - digits..];
        format!("{}.{}", int_part, frac_part)
    }
}

pub fn chudnovsky_pi(digits: usize) -> String {
    let terms = digits.div_ceil(14) + 2;
    let result = chudnovsky_bs(0, terms as i64);
    let scale = BigInt::from(10u64).pow(digits as u32);
    let sqrt_arg = BigInt::from(10005u64) * &scale * &scale;
    let sqrt_10005 = integer_sqrt(&sqrt_arg);
    let c = BigInt::from(426880u64) * sqrt_10005;
    let pi_scaled = c * result.q / result.t;
    format_pi(&pi_scaled, digits)
}

pub fn ramanujan_pi(digits: usize) -> String {
    let terms = digits.div_ceil(8) + 2;
    let result = ramanujan_bs(0, terms as i64);
    let scale = BigInt::from(10u64).pow(digits as u32);
    let two = BigInt::from(2u64);
    let sqrt_arg = BigInt::from(2u64) * &scale * &scale;
    let sqrt_2 = integer_sqrt(&sqrt_arg);
    let two_root2 = &two * sqrt_2;
    let nine801 = BigInt::from(9801u64);
    let pi_scaled = nine801 * result.q * &scale * &scale / (two_root2 * result.t);
    format_pi(&pi_scaled, digits)
}

#[allow(dead_code)]
pub fn chudnovsky_pi_f64(terms: usize) -> f64 {
    let c = 426880.0 * (10005.0_f64).sqrt();
    let mut s = 0.0_f64;
    for k in 0..terms {
        let num = (13591409 + 545140134 * k) as f64;
        let num_fact = factorial_f64(6 * k);
        let den1 = factorial_f64(3 * k);
        let den2 = factorial_f64(k).powi(3);
        let den3 = (-640320.0_f64).powi((3 * k) as i32);
        s += num * num_fact / (den1 * den2 * den3);
    }
    c / s
}

#[allow(dead_code)]
pub fn ramanujan_pi_f64(terms: usize) -> f64 {
    let c = (2.0 * std::f64::consts::SQRT_2) / 9801.0;
    let mut s = 0.0_f64;
    for k in 0..terms {
        let num = (1103 + 26390 * k) as f64;
        let num_fact = factorial_f64(4 * k);
        let den = factorial_f64(k).powi(4) * (396.0_f64).powi((4 * k) as i32);
        s += num * num_fact / den;
    }
    1.0 / (c * s)
}

#[allow(dead_code)]
fn factorial_f64(n: usize) -> f64 {
    (1..=n).fold(1.0, |a, b| a * b as f64)
}

pub fn benchmark_chudnovsky(digits: usize) -> String {
    let start = Instant::now();
    let pi = chudnovsky_pi(digits);
    let elapsed = start.elapsed();
    eprintln!("MATH_TIME: {:.4}ms", elapsed.as_secs_f64() * 1000.0);
    pi
}

pub fn benchmark_ramanujan(digits: usize) -> String {
    let start = Instant::now();
    let pi = ramanujan_pi(digits);
    let elapsed = start.elapsed();
    eprintln!("MATH_TIME: {:.4}ms", elapsed.as_secs_f64() * 1000.0);
    pi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chudnovsky_f64_small() {
        let pi = chudnovsky_pi_f64(2);
        assert!((pi - std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn test_ramanujan_f64_small() {
        let pi = ramanujan_pi_f64(3);
        assert!((pi - std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn test_chudnovsky_100_digits() {
        let pi = chudnovsky_pi(100);
        assert!(pi.starts_with("3."));
        assert_eq!(pi.len(), 102);
        assert_eq!(&pi[..20], "3.141592653589793238");
    }

    #[test]
    fn test_ramanujan_100_digits() {
        let pi = ramanujan_pi(100);
        assert!(pi.starts_with("3."));
        assert_eq!(pi.len(), 102);
        assert_eq!(&pi[..20], "3.141592653589793238");
    }

    #[test]
    fn test_chudnovsky_500_digits() {
        let pi = chudnovsky_pi(500);
        assert!(pi.starts_with("3."));
        let pi_trimmed: String = pi.chars().filter(|&c| c != '.').collect();
        let expected_start = "31415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679";
        assert!(pi_trimmed.starts_with(expected_start));
    }
}
