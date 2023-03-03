use std::slice::range;
// Bring in some tools for using finite fiels
use ff::{PrimeField};

// We'll use these interfaces to construct our circuit.
use bellman::{Circuit, ConstraintSystem, LinearCombination, SynthesisError, Variable};
use bellman::domain::Scalar;

/*pub struct RangeA {
    a: Option<u64>,
    b: Option<u64>,
    w:Option<u64>,
    wArray:Option<[u64;4]>,
    less_or_equal:Option<u64>,
    less:Option<u64>,
    not_all_zeros:Option<u64>,
    crArray:Option<[u64;4]>,
}*/

pub struct Range {
    a: Variable,
    b: Variable,
    w:Variable,
    wArray:Vec<Variable>,
    crArray:Vec<Variable>,
    not_all_zeros:Variable,
    less_or_equal:u64,
    less:u64,

}

impl Range{
    pub fn alloc<Scalar, CS>(mut cs: CS, a:Option<(u64,bool)>,b: Option<(u64,bool)>,w:Option<(u64,bool)>,wArray:Option<([u64;4],bool)>,crArray:Option<([u64;4],bool)>,less_or_equal:u64,less:u64,not_all_zeros:Option<(u64,bool)>) -> Result<Self, SynthesisError>
        where
            Scalar: PrimeField,
            CS: ConstraintSystem<Scalar>,
    {
        let a_var = Scalar::from(a.unwrap().0);
        let b_var = Scalar::from(b.unwrap().0);
        let w_var = Scalar::from(w.unwrap().0);
        let mut a_v=match a.unwrap().1 {
            true =>cs.alloc(|| "a", ||  Ok(a_var))?,
            false =>cs.alloc_input(|| "a", ||  Ok(a_var))?,
        };
        let mut b_v=match b.unwrap().1 {
            true =>cs.alloc(|| "b", || Ok(b_var))?,
            false =>cs.alloc_input(|| "b", || Ok(b_var))?,
        };
        let mut w_v=match w.unwrap().1 {
            true =>cs.alloc(|| "w", || Ok(w_var))?,
            false =>cs.alloc_input(|| "w", || Ok(w_var))?,
        };

        let not_all_zeros_var = Scalar::from(not_all_zeros.unwrap().0);
        let mut not_all_zeros_v=match not_all_zeros.unwrap().1 {
            true =>cs.alloc(|| "not_all_zeros", || Ok(not_all_zeros_var))?,
            false =>cs.alloc_input(|| "not_all_zeros", || Ok(not_all_zeros_var))?,
        };
        let mut wArray_var= vec![];
        for i in 0 .. wArray.unwrap().0.len(){
            let mut wArray_v=match wArray.unwrap().1 {
                true =>cs.alloc(||"",||Ok(Scalar::from(*wArray.unwrap().0.get(i).unwrap()))),
                false =>cs.alloc_input(||"",||Ok(Scalar::from(*wArray.unwrap().0.get(i).unwrap()))),
            };
            wArray_var.push(wArray_v.unwrap());
        }
        let mut crArray_var= vec![];
        for i in 0 .. crArray.unwrap().0.len() {
            let mut cArray_v=match crArray.unwrap().1 {
                true =>cs.alloc(||"",||Ok(Scalar::from(*crArray.unwrap().0.get(i).unwrap()))),
                false =>cs.alloc_input(||"",||Ok(Scalar::from(*crArray.unwrap().0.get(i).unwrap()))),
            };
        }
        Ok(Range {
            a: a_v,
            b: b_v,
            w:w_v,
            wArray:wArray_var,
            less_or_equal:less_or_equal,
            less:less,
            not_all_zeros:not_all_zeros_v,
            crArray:crArray_var,
        })
    }

pub fn less_or_equal<Scalar, CS>(mut cs: CS,a:(u64,bool),b:(u64,bool)) -> Result<(), SynthesisError>
    where
        Scalar: PrimeField,
        CS: ConstraintSystem<Scalar>,
{
    let namespace = cs.namespace("aa" |);

    let w= 1<<(64-1u64)+b-a;
    let r=Range::alloc(namespace,Option(a),Option(b),Option((w,a.1&b.1)),x,y,1,1,Option(,a.1&b.1)).expect("");

    let namespace = cs.namespace("aa" |);
    Range::range(cs,&r);
}

pub fn less<Scalar, CS>(mut cs: CS, input: &Range, ) -> Result<(), SynthesisError>
    where
        Scalar: PrimeField,
        CS: ConstraintSystem<Scalar>,
{
    Ok(())
}

pub fn large_or_equal<Scalar, CS>(
    mut cs: CS,
    input: &Range,
) -> Result<(), SynthesisError>
    where
        Scalar: PrimeField,
        CS: ConstraintSystem<Scalar>,
{
    Ok(())
}

pub fn large<Scalar, CS>(
    mut cs: CS,
    input: &Range,
) -> Result<(), SynthesisError>
    where
        Scalar: PrimeField,
        CS: ConstraintSystem<Scalar>,
{
    Ok(())
}

pub fn range<Scalar, CS>(mut cs: CS, input: &Range) -> Result<(), SynthesisError> where Scalar: PrimeField, CS: ConstraintSystem<Scalar>,
{
    let a=input.a;
    let b=input.b;
    let n=input.wArray.len();
    let exp2n =1<<(n-1u64);
    let wArray=*input.wArray;
    let crArray=*input.crArray;
    let not_all_zeros=input.not_all_zeros;
    let less_or_equal=input.less_or_equal;
    let less=input.less;
    cs.enforce(
        || "w=2^n+b-a",
        |lc| lc + w,
        |lc| lc + CS::one(),
        |lc| lc+(Scalar::from(exp2n), CS::one())+b-a,
    );

    let mut lct = LinearCombination::<Scalar>::zero();
    for i in 0..wArray.len(){
        lct = lct + (Scalar::from(1<<i), wArray[i]);
    }
    lct = lct -w;
    cs.enforce(
        || "2^0*w0+.......-w=0",
        |_| lct,
        |lc| lc + CS::one(),
        |lc| lc,
    );

    for i in 0..wArray.len() {
        cs.enforce(
            || "w0(1-w0)=0",
            |lc| lc + wArray[i],
            |lc| lc + CS::one()-wArray[i],
            |lc| lc,
        );
    }

    cs.enforce(
        || "w0=cr0",
        |lc| lc + wArray[0],
        |lc| lc + CS::one(),
        |lc| lc+crArray[0],
    );

    for i in 1..crArray_var.len() {
        cs.enforce(
            || "(cr_(i-1)-1)(wi-1)=1-cr_i",
            |lc| lc + crArray[i-1]-CS::one(),
            |lc| lc + wArray[i]-CS::one(),
            |lc| lc+CS::one()- crArray[i],
        );
    }

    cs.enforce(
        || "not_all_zeros=cr_n",
        |lc| lc + not_all_zeros,
        |lc| lc + CS::one(),
        |lc| lc+crArray[crArray.len()-1],
    );

    cs.enforce(
        || "wn=less_or_equal*wn",
        |lc| lc + wArray[wArray.len()-1],
        |lc| lc + Scalar::from(less_or_equal),
        |lc| lc+wArray[wArray.len()-1],
    );

    cs.enforce(
        || "wn*less_or_equal=less",
        |lc| lc + wArray[wArray.len()-1],
        |lc| lc + not_all_zeros,
        |lc| lc+Scalar::from(less),
    );
    Ok(())
}
}
