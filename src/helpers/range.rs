// Bring in some tools for using finite fiels
use ff::PrimeField;

// We'll use these interfaces to construct our circuit.
use bellman::{ConstraintSystem, LinearCombination, SynthesisError, Variable};

///Here we provide some functions to create different types of integer data comparison circuits.
///Mainly contains 'less' and 'less_or_equal' function.Other comparison logic can be realized through certain transformations.
struct Range {
    a: Variable,
    b: Variable,
    w: Variable,
    w_array: Vec<Variable>,
    cr_array: Vec<Variable>,
    not_all_zeros: Variable,
    less_or_equal: u64,
    less: u64,
}

trait Alloc<U> {
    fn alloc<Scalar, CS>(
        cs: CS,
        a: (U, bool),
        b: (U, bool),
        w: (U, bool),
        w_array: (Vec<U>, bool),
        cr_array: (Vec<U>, bool),
        less_or_equal: u64,
        less: u64,
        not_all_zeros: (U, bool),
    ) -> Result<Range, SynthesisError>
    where
        Scalar: PrimeField,
        CS: ConstraintSystem<Scalar>;
}

impl<U: Copy> Alloc<U> for Range
where
    u64: From<U>,
{
    fn alloc<Scalar, CS>(
        mut cs: CS,
        a: (U, bool),
        b: (U, bool),
        w: (U, bool),
        w_array: (Vec<U>, bool),
        cr_array: (Vec<U>, bool),
        less_or_equal: u64,
        less: u64,
        not_all_zeros: (U, bool),
    ) -> Result<Self, SynthesisError>
    where
        Scalar: PrimeField,
        CS: ConstraintSystem<Scalar>,
    {
        let a_var = Scalar::from(a.0.into());
        let b_var = Scalar::from(b.0.into());
        let w_var = Scalar::from(w.0.into());
        let a_v = match a.1 {
            true => cs.alloc_input(|| "a", || Ok(a_var))?,
            false => cs.alloc(|| "a", || Ok(a_var))?,
        };
        let b_v = match b.1 {
            true => cs.alloc_input(|| "b", || Ok(b_var))?,
            false => cs.alloc(|| "b", || Ok(b_var))?,
        };
        let w_v = match w.1 {
            true => cs.alloc_input(|| "w", || Ok(w_var))?,
            false => cs.alloc(|| "w", || Ok(w_var))?,
        };

        let not_all_zeros_var = Scalar::from(not_all_zeros.0.into());
        let not_all_zeros_v = match not_all_zeros.1 {
            true => cs.alloc_input(|| "not_all_zeros", || Ok(not_all_zeros_var))?,
            false => cs.alloc(|| "not_all_zeros", || Ok(not_all_zeros_var))?,
        };
        let mut w_array_v = vec![];
        for i in 0..w_array.0.len() {
            let w_var = *w_array.0.get(i).unwrap();
            let w_v = match w_array.1 {
                true => cs.alloc_input(
                    || format!("w_array_{}", i),
                    || Ok(Scalar::from(w_var.into())),
                ),
                false => cs.alloc(
                    || format!("w_array_{}", i),
                    || Ok(Scalar::from(w_var.into())),
                ),
            };
            w_array_v.push(w_v.unwrap());
        }
        let mut cr_array_v = vec![];
        for i in 0..cr_array.0.len() {
            let cr_var = *cr_array.0.get(i).unwrap();
            let cr_v = match cr_array.1 {
                true => cs.alloc_input(
                    || format!("cr_array_{}", i),
                    || Ok(Scalar::from(cr_var.into())),
                ),
                false => cs.alloc(
                    || format!("cr_array_{}", i),
                    || Ok(Scalar::from(cr_var.into())),
                ),
            };
            cr_array_v.push(cr_v.unwrap());
        }

        Ok(Range {
            a: a_v,
            b: b_v,
            w: w_v,
            w_array: w_array_v,
            cr_array: cr_array_v,
            less_or_equal: less_or_equal,
            less: less,
            not_all_zeros: not_all_zeros_v,
        })
    }
}

impl Range {
    pub fn execute<Scalar, CS>(mut cs: CS, input: &Range) -> Result<(), SynthesisError>
    where
        Scalar: PrimeField,
        CS: ConstraintSystem<Scalar>,
    {
        let a = input.a;
        let b = input.b;
        let n = input.w_array.len();
        let w = input.w;
        let exp2n = 1 << (n - 1);
        let w_array = &input.w_array;
        let cr_array = &input.cr_array;
        let not_all_zeros = input.not_all_zeros;
        let less_or_equal = input.less_or_equal;
        let less = input.less;
        cs.enforce(
            || "w=2^n+b-a",
            |lc| lc + w,
            |lc| lc + CS::one(),
            |lc| lc + (Scalar::from(exp2n), CS::one()) + b - a,
        );

        let mut lct = LinearCombination::<Scalar>::zero();
        for i in 0..w_array.len() {
            lct = lct + (Scalar::from(1 << i), w_array[i]);
        }
        lct = lct - w;
        cs.enforce(
            || "2^0*w_0+.......-w=0",
            |_| lct,
            |lc| lc + CS::one(),
            |lc| lc,
        );

        for i in 0..w_array.len() {
            cs.enforce(
                || format!("w_{0}(1-w_{0})=0", i),
                |lc| lc + w_array[i],
                |lc| lc + CS::one() - w_array[i],
                |lc| lc,
            );
        }

        cs.enforce(
            || "w_0=cr_0",
            |lc| lc + w_array[0],
            |lc| lc + CS::one(),
            |lc| lc + cr_array[0],
        );

        for i in 1..cr_array.len() {
            cs.enforce(
                || format!("(cr_{0}-1)(w_{1}-1)=1-cr_{1}", i - 1, i),
                |lc| lc + cr_array[i - 1] - CS::one(),
                |lc| lc + w_array[i] - CS::one(),
                |lc| lc + CS::one() - cr_array[i],
            );
        }

        cs.enforce(
            || "not_all_zeros=cr_n",
            |lc| lc + not_all_zeros,
            |lc| lc + CS::one(),
            |lc| lc + cr_array[cr_array.len() - 1],
        );

        cs.enforce(
            || "w_n=less_or_equal*1",
            |lc| lc + CS::one(),
            |lc| lc + (Scalar::from(less_or_equal), CS::one()),
            |lc| lc + w_array[w_array.len() - 1],
        );

        cs.enforce(
            || "less*not_all_zeros=less",
            |lc| lc + (Scalar::from(less), CS::one()),
            |lc| lc + not_all_zeros,
            |lc| lc + (Scalar::from(less), CS::one()),
        );
        Ok(())
    }
}

fn comparison_u64<Scalar, CS>(
    mut cs: CS,
    name: &str,
    a: (u64, bool),
    b: (u64, bool),
    less_or_equal: u64,
    less: u64,
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    let w = ((1 << (64 - 1u64)) + (b.0 - a.0)) as u64;
    let mut w_array = [0].repeat(64);
    let mut i = 0;
    let mut values = w.clone();
    while i < w_array.len() {
        w_array[i] = values & 1;
        values >>= 1;
        i += 1;
    }
    let mut c_array = [0].repeat(63);
    c_array[0] = w_array[0];
    let mut j = 1;
    while j < c_array.len() {
        c_array[j] = w_array[j] | c_array[j - 1];
        j += 1;
    }
    let not_all_zeros = c_array.last().unwrap().clone();
    let r = Range::alloc(
        cs.namespace(|| format!("{0}_alloc", name)),
        a,
        b,
        (w, a.1 & b.1),
        (w_array, a.1 & b.1),
        (c_array, a.1 & b.1),
        less_or_equal,
        less,
        (not_all_zeros, a.1 & b.1),
    )
    .expect(&format!("{0}_alloc_error", name));
    return Range::execute(cs.namespace(|| name), &r);
}

fn comparison_u32<Scalar, CS>(
    mut cs: CS,
    name: &str,
    a: (u32, bool),
    b: (u32, bool),
    less_or_equal: u64,
    less: u64,
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    let w = ((1 << (32 - 1u32)) + (b.0 - a.0)) as u32;
    let mut w_array = [0].repeat(32);
    let mut i = 0;
    let mut values = w.clone();
    while i < w_array.len() {
        w_array[i] = values & 1;
        values >>= 1;
        i += 1;
    }
    let mut c_array = [0].repeat(31);
    c_array[0] = w_array[0];
    let mut j = 1;
    while j < c_array.len() {
        c_array[j] = w_array[j] | c_array[j - 1];
        j += 1;
    }
    let not_all_zeros = c_array.last().unwrap().clone();
    let r = Range::alloc(
        cs.namespace(|| format!("{0}_alloc", name)),
        a,
        b,
        (w, a.1 & b.1),
        (w_array, a.1 & b.1),
        (c_array, a.1 & b.1),
        less_or_equal,
        less,
        (not_all_zeros, a.1 & b.1),
    )
    .expect(&format!("{0}_alloc_error", name));
    return Range::execute(cs.namespace(|| name), &r);
}

fn comparison_u16<Scalar, CS>(
    mut cs: CS,
    name: &str,
    a: (u16, bool),
    b: (u16, bool),
    less_or_equal: u64,
    less: u64,
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    let w = ((1 << (16 - 1u16)) + (b.0 - a.0)) as u16;
    let mut w_array = [0].repeat(16);
    let mut i = 0;
    let mut values = w.clone();
    while i < w_array.len() {
        w_array[i] = values & 1;
        values >>= 1;
        i += 1;
    }
    let mut c_array = [0].repeat(15);
    c_array[0] = w_array[0];
    let mut j = 1;
    while j < c_array.len() {
        c_array[j] = w_array[j] | c_array[j - 1];
        j += 1;
    }
    let not_all_zeros = c_array.last().unwrap().clone();
    let r = Range::alloc(
        cs.namespace(|| format!("{0}_alloc", name)),
        a,
        b,
        (w, a.1 & b.1),
        (w_array, a.1 & b.1),
        (c_array, a.1 & b.1),
        less_or_equal,
        less,
        (not_all_zeros, a.1 & b.1),
    )
    .expect(&format!("{0}_alloc_error", name));
    return Range::execute(cs.namespace(|| name), &r);
}

fn comparison_u8<Scalar, CS>(
    mut cs: CS,
    name: &str,
    a: (u8, bool),
    b: (u8, bool),
    less_or_equal: u64,
    less: u64,
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    let w = ((1 << (8 - 1u8)) + (b.0 - a.0)) as u8;
    let mut w_array = [0].repeat(8);
    let mut i = 0;
    let mut values = w.clone();
    while i < w_array.len() {
        w_array[i] = values & 1;
        values >>= 1;
        i += 1;
    }
    let mut c_array = [0].repeat(7);
    c_array[0] = w_array[0];
    let mut j = 1;
    while j < c_array.len() {
        c_array[j] = w_array[j] | c_array[j - 1];
        j += 1;
    }
    let not_all_zeros = c_array.last().unwrap().clone();
    let r = Range::alloc(
        cs.namespace(|| format!("{0}_alloc", name)),
        a,
        b,
        (w, a.1 & b.1),
        (w_array, a.1 & b.1),
        (c_array, a.1 & b.1),
        less_or_equal,
        less,
        (not_all_zeros, a.1 & b.1),
    )
    .expect(&format!("{0}_alloc_error", name));
    return Range::execute(cs.namespace(|| name), &r);
}

pub fn less_or_equal_u64<Scalar, CS>(
    cs: CS,
    a: (u64, bool),
    b: (u64, bool),
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u64(cs, "less_or_equal", a, b, 1, 0)
}

pub fn less_or_equal_u32<Scalar, CS>(
    cs: CS,
    a: (u32, bool),
    b: (u32, bool),
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u32(cs, "less_or_equal", a, b, 1, 0)
}

pub fn less_or_equal_u16<Scalar, CS>(
    cs: CS,
    a: (u16, bool),
    b: (u16, bool),
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u16(cs, "less_or_equal", a, b, 1, 0)
}

pub fn less_or_equal_u8<Scalar, CS>(
    cs: CS,
    a: (u8, bool),
    b: (u8, bool),
) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u8(cs, "less_or_equal", a, b, 1, 0)
}

pub fn less_u64<Scalar, CS>(cs: CS, a: (u64, bool), b: (u64, bool)) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u64(cs, "less", a, b, 1, 1)
}

pub fn less_u32<Scalar, CS>(cs: CS, a: (u32, bool), b: (u32, bool)) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u32(cs, "less", a, b, 1, 1)
}

pub fn less_u16<Scalar, CS>(cs: CS, a: (u16, bool), b: (u16, bool)) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u16(cs, "less", a, b, 1, 1)
}

pub fn less_u8<Scalar, CS>(cs: CS, a: (u8, bool), b: (u8, bool)) -> Result<(), SynthesisError>
where
    Scalar: PrimeField,
    CS: ConstraintSystem<Scalar>,
{
    comparison_u8(cs, "less", a, b, 1, 1)
}

#[cfg(test)]
mod test {
    use crate::helpers::range::{
        less_or_equal_u16, less_or_equal_u32, less_or_equal_u64, less_or_equal_u8, less_u16,
        less_u32, less_u64, less_u8,
    };
    use bellman::{gadgets::test::TestConstraintSystem, ConstraintSystem};
    use bls12_381::Scalar;

    #[test]
    fn test_less_or_equal_u64() {
        // pub left
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_or_equal_u64(cs.namespace(|| "less_or_equal_u64"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        // no pub
        cs = TestConstraintSystem::<Scalar>::new();
        result = less_or_equal_u64(cs.namespace(|| "less_or_equal_u64"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }

    #[test]
    fn test_less_or_equal_u32() {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_or_equal_u32(cs.namespace(|| "less_or_equal_u32"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        cs = TestConstraintSystem::<Scalar>::new();
        result = less_or_equal_u32(cs.namespace(|| "less_or_equal_u32"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }

    #[test]
    fn test_less_or_equal_u16() {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_or_equal_u16(cs.namespace(|| "less_or_equal_u16"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        cs = TestConstraintSystem::<Scalar>::new();
        result = less_or_equal_u16(cs.namespace(|| "less_or_equal_u16"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }

    #[test]
    fn test_less_or_equal_u8() {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_or_equal_u8(cs.namespace(|| "less_or_equal_u8"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        cs = TestConstraintSystem::<Scalar>::new();
        result = less_or_equal_u8(cs.namespace(|| "less_or_equal_u8"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }

    #[test]
    fn test_less_u64() {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_u64(cs.namespace(|| "less_u64"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        cs = TestConstraintSystem::<Scalar>::new();
        result = less_u64(cs.namespace(|| "less_u64"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }

    #[test]
    fn test_less_u32() {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_u32(cs.namespace(|| "less_u32"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        cs = TestConstraintSystem::<Scalar>::new();
        result = less_u32(cs.namespace(|| "less_u32"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }

    #[test]
    fn test_less_u16() {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_u16(cs.namespace(|| "less_u16"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        cs = TestConstraintSystem::<Scalar>::new();
        result = less_u16(cs.namespace(|| "less_u16"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }

    #[test]
    fn test_less_u8() {
        let mut cs = TestConstraintSystem::<Scalar>::new();
        let mut result = less_u8(cs.namespace(|| "less_u8"), (1, true), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[Scalar::from(1)]));

        cs = TestConstraintSystem::<Scalar>::new();
        result = less_u8(cs.namespace(|| "less_u8"), (1, false), (2, false));
        assert!(result.is_ok());
        assert!(cs.is_satisfied());
        assert!(cs.verify(&[]));
    }
}
