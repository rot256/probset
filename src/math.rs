use num::bigint::Sign;
use num::rational::Ratio;
use num::traits::cast::FromPrimitive;
use num::BigInt;
use num::BigUint;
use num::Integer;
use num::Zero;

use core::ops::Shr;

use std::cmp;

pub type Int = BigInt;
pub type Rat = Ratio<Int>;

fn exp2u(e: u64) -> BigUint {
    let n = 1 + (e / 32) as usize;
    let mut v: Vec<u32> = vec![0; n];
    v[n - 1] = 1 << (e % 32);
    BigUint::from_slice(&v[..])
}

/// Helper to quickly calculate 2^e.
///
/// Arguments:
///
/// - e: Exponent
///
/// Returns:
///
/// Integer representing 2^e
///
fn exp2(e: u64) -> Int {
    let n = 1 + (e / 32) as usize;
    let mut v: Vec<u32> = vec![0; n];
    v[n - 1] = 1 << (e % 32);
    Int::from_slice(Sign::Plus, &v[..])
}

fn trucr(a: Rat, r: u64) -> Rat {
    let n = (a * exp2(r)).to_integer();
    Rat::new(n, exp2(r))
}

/// Rational power.
///
/// Arguments:
///
/// - base:
/// - exp: Exponent (non-negative rational)
/// - prec: Bits of precision
///
/// Returns:
///
/// An approximation for rational base^exp with r bits of precision.
pub fn pow(base: Rat, exp: Rat, prec: u64) -> Option<Rat> {
    fn rec(base: &Rat, p: BigUint, d: u64, r: u64) -> Rat {
        println!("{}", p);
        // base case (p == 0)
        if p.is_zero() {
            return Rat::from_u64(1).unwrap();
        }

        // induction step
        if p.is_even() {
            rec(base, p.shr(1), d - 1, r)
        } else {
            let rem = nroot(base.clone(), exp2u(d), r);
            rem * rec(base, p.shr(1), d - 1, r)
        }
    }

    let powr = (exp * exp2(prec)).to_integer().to_biguint()?;

    Some(rec(&base, powr, prec, prec))
}

/// Integer power.
///
/// Arguments:
///
/// - base: Rational base
/// - exp: Exponent (unsigned bignum)
///
/// Returns:
///
/// The rational base^exp
///
fn powi(mut base: Rat, exp: &BigUint) -> Rat {
    let mut acc = Rat::from_integer(Int::new(Sign::Plus, vec![1]));
    for byte in exp.to_bytes_le() {
        for bit in 0..7 {
            if (byte >> bit) & 1 == 1 {
                acc *= base.clone();
            }
            base = base.clone() * base;
        }
    }
    acc
}

/// Computes the n'th root of a positive rational
///
/// - A:
/// - n:
/// - r: resolution in bits:
///      how many bits of accuracy is desired beyond "decimal" point.
fn nroot(a: Rat, n: BigUint, r: u64) -> Rat {
    fn trucr(a: Rat, r: u64) -> Rat {
        let n = (a * exp2(r)).to_integer();
        Rat::new(n, exp2(r))
    }

    let n1 = n.clone() - BigUint::new(vec![1]);
    let n = Int::from_biguint(Sign::Plus, n);
    let mut x = Rat::from_integer(Int::from_slice(Sign::Plus, &[1]));

    loop {
        let x_n = powi(x.clone(), &n1);
        let x_d = (a.clone() / x_n - x.clone()) / n.clone();
        let x_t = trucr(x.clone() + x_d.clone(), r);
        if x_t == x {
            break x_t;
        }
        x = x_t;
    }
}

pub fn decimal(mut rat: Rat, r: usize) -> String {
    let mut chr: Vec<char> = Vec::with_capacity(r + 32);
    let digit: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

    // shift and round
    for _ in 0..r {
        rat = rat * Rat::from_integer(Int::from_slice(Sign::Plus, &[10]));
    }
    let num = rat.round().to_integer();

    // extract digits
    let (sign, digits) = num.to_radix_le(10);

    for i in 0..cmp::max(digits.len(), r + 1) {
        if i == r {
            chr.push('.');
        }
        match digits.get(i) {
            Some(d) => {
                chr.push(digit[*d as usize]);
            }
            None => chr.push('0'),
        }
    }

    if sign == Sign::Minus {
        chr.push('-');
    }

    chr.iter().rev().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powi() {
        // Tests generated using SageMath:
        //
        // >>> N(a^e, bits)
        let tests = vec![
            /*
            (
                100,
                Rational::new(
                    Integer::new(Sign::Plus, vec![1]),
                    Integer::new(Sign::Plus, vec![2]),
                ),
                Rational::new(
                    Integer::new(Sign::Plus, vec![2]),
                    Integer::new(Sign::Plus, vec![43]),
                ),
                "0.9682747457268316612635090387308222376425350457206217124267393890075195068030503706961430269493507673"
            ),
            */
            (
                100,
                Rat::new(
                    Int::new(Sign::Plus, vec![1]),
                    Int::new(Sign::Plus, vec![2]),
                ),
                Rat::new(
                    Int::new(Sign::Plus, vec![2147483659]),
                    Int::new(Sign::Plus, vec![2147485999]),
                ),
                "0.9682747457268316612635090387308222376425350457206217124267393890075195068030503706961430269493507673"
            ),
        ];
        for (n, a, e, res) in tests {
            let r = pow(a, e, 512); // 2^4 = 16 > 10
            assert_eq!(decimal(r.unwrap(), n), res);
        }
    }
}
