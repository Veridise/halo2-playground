use std::marker::PhantomData;
use halo2_proofs::{arithmetic::FieldExt, circuit::*, plonk::*, poly::Rotation};

#[derive(Debug, Clone)]
struct IteConfig {
    pub oneval: Column<Fixed>,
    pub cond: Column<Advice>,
    pub thenval: Column<Advice>,
    pub elseval: Column<Advice>,
    pub outval: Column<Advice>,
    pub selector: Selector,
    pub instance: Column<Instance>,
}

#[derive(Debug, Clone)]
struct IteChip<F: FieldExt> {
    config: IteConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> IteChip<F> {
    pub fn construct(config: IteConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> IteConfig {
        let oneval = meta.fixed_column();
        let cond = meta.advice_column();
        let thenval = meta.advice_column();
        let elseval = meta.advice_column();
        let outval = meta.advice_column();
        let selector = meta.selector();
        let instance = meta.instance_column();

        // meta.enable_equality(oneval);
        meta.enable_constant(oneval);
        meta.enable_equality(cond);
        meta.enable_equality(thenval);
        meta.enable_equality(elseval);
        meta.enable_equality(outval);
        meta.enable_equality(instance);

        // meta.create_gate("ite", |meta| {
        //     //
        //     // oneval | cond | thenval | elseval | outval | selector
        //     //   n        c       t         e        o         s
        //     //
        //     let s = meta.query_selector(selector);
        //     let n = meta.query_fixed(oneval, Rotation::cur());
        //     let c = meta.query_advice(cond, Rotation::cur());
        //     let t = meta.query_advice(thenval, Rotation::cur());
        //     let e = meta.query_advice(elseval, Rotation::cur());
        //     let o = meta.query_advice(outval, Rotation::cur());
        //     let one = Expression::Constant(F::one());
        //     let zero = Expression::Constant(F::zero());
        //     vec![
        //         s * ( c.clone() * t + (one - c.clone()) * e - o )
        //         // s * (e - o)
        //         // zero
        //     ]
        // });

        IteConfig {
            oneval,
            cond,
            thenval,
            elseval,
            outval,
            selector,
            instance,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
    ) -> Result<AssignedCell<F, F>, Error> {
        layouter.assign_region(
            || "ite row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                let cond_cell = region.assign_advice_from_instance(
                    || "cond",
                    self.config.instance,
                    0,
                    self.config.cond,
                    0,
                )?;

                let thenval_cell = region.assign_advice_from_instance(
                    || "thenval",
                    self.config.instance,
                    1,
                    self.config.thenval,
                    0,
                )?;

                let elseval_cell = region.assign_advice_from_instance(
                    || "elseval",
                    self.config.instance,
                    2,
                    self.config.elseval,
                    0,
                )?;

                let one_cell = region.assign_fixed(
                    || "oneval",
                    self.config.oneval,
                    0,
                    || -> Value<F> {Value::known(F::one())},
                )?;

                let outval_cell = region.assign_advice(
                    || "outval",
                    self.config.outval,
                    0,
                    // || cond_cell.value().copied() * thenval_cell.value().copied() + (one_cell.value().copied() - cond_cell.value().copied()) * elseval_cell.value().copied()
                    // || if F::one() == cond_cell.value().copied() { thenval_cell.value().copied() } else { elseval_cell.value().copied() }
                    || elseval_cell.value().copied()
                )?;

                Ok(outval_cell)
            },
        )
    }

    pub fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct MyCircuit<F>(PhantomData<F>);

impl<F: FieldExt> Circuit<F> for MyCircuit<F> {
    type Config = IteConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        IteChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = IteChip::construct(config);

        let outval_cell = chip.assign(layouter.namespace(|| "ite row"))?;

        chip.expose_public(layouter.namespace(|| "outval"), &outval_cell, 0)?;

        Ok(())
    }
}

fn main() {
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    // ANCHOR: test-circuit
    // The number of rows in our circuit cannot exceed 2^k. Since our example
    // circuit is very small, we can pick a very small value here.
    let k = 4;

    let cond = Fp::from(1);
    let thenval = Fp::from(2);
    let elseval = Fp::from(3);
    // let outval = if cond == Fp::from(1) {thenval} else {elseval};

    let circuit = MyCircuit(PhantomData);

    let public_input = vec![cond, thenval, elseval];

    println!("# start.");
    let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
    prover.assert_satisfied();
    println!("# done.");

    // ANCHOR_END: test-circuit
}