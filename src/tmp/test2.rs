use std::marker::PhantomData;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, Instance, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

#[derive(Debug, Clone)]
struct FunctionConfig {
    selector: Selector,
    cond: Column<Advice>,
    thenval: Column<Advice>,
    elseval: Column<Advice>,
    output: Column<Advice>,
    instance: Column<Instance>,
}

#[derive(Debug, Clone)]
struct FunctionChip<F: FieldExt> {
    config: FunctionConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> FunctionChip<F> {
    pub fn construct(config: FunctionConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> FunctionConfig {
        let selector = meta.selector();
        let cond = meta.advice_column();
        let thenval = meta.advice_column();
        let elseval = meta.advice_column();
        let output = meta.advice_column();
        let instance = meta.instance_column();

        meta.create_gate("f(a, b, c) = if a == b {c} else {a - b}", |meta| {
            let s = meta.query_selector(selector);
            let cond = meta.query_advice(cond, Rotation::cur());
            let thenval = meta.query_advice(thenval, Rotation::cur());
            let elseval = meta.query_advice(elseval, Rotation::cur());
            let output = meta.query_advice(output, Rotation::cur());

            let one = Expression::Constant(F::one());

            vec![
                s * ( cond.clone() * thenval + (one - cond.clone()) * elseval - output )
            ]
        });

        FunctionConfig {
            selector,
            cond,
            thenval,
            elseval,
            output,
            instance,
        }
    }

    pub fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        cond: F,
        thenval: F,
        elseval: F,
    ) -> Result<AssignedCell<F, F>, Error> {
        layouter.assign_region(
            || "f(cond, thenval, elseval) = if cond == 1 {thenval} else {elseval}",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;
                region.assign_advice(|| "cond", self.config.cond, 0, || Value::known(cond))?;
                region.assign_advice(|| "thenval", self.config.thenval, 0, || Value::known(thenval))?;
                region.assign_advice(|| "elseval", self.config.elseval, 0, || Value::known(elseval))?;

                let output = if cond == F::one() { thenval } else { elseval };
                // let output = cond;
                region.assign_advice(|| "output", self.config.output, 0, || Value::known(output))
            },
        )
    }
}

#[derive(Default)]
struct FunctionCircuit<F> {
    cond: F,
    thenval: F,
    elseval: F,
}

impl<F: FieldExt> Circuit<F> for FunctionCircuit<F> {
    type Config = FunctionConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        FunctionChip::configure(meta)
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = FunctionChip::construct(config);
        let c = chip.assign(layouter, self.cond, self.thenval, self.elseval)?;
        chip.expose_public(layouter.namespace(|| "instance"), &c, 0)?;
        Ok(())
    }
}

fn main() {
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    println!("# Example0.");

    // ANCHOR: test-circuit
    // The number of rows in our circuit cannot exceed 2^k. Since our example
    // circuit is very small, we can pick a very small value here.
    let k = 4;

    let cond = Fp::from(1);
    let thenval = Fp::from(2);
    let elseval = Fp::from(3);

    let output = if cond == Fp::from(1) {thenval} else {elseval};

    let circuit = FunctionCircuit {
        cond: Fp::from(cond),
        thenval: Fp::from(thenval),
        elseval: Fp::from(elseval),
    };

    println!("# Prover is expected to return OK.");
    let public_inputs = vec![];
    let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
    prover.assert_satisfied();
    println!("# Done.");

    // ANCHOR_END: test-circuit
}