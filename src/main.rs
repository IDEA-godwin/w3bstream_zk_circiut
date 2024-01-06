use halo2_curves::ff::Field;
use halo2_proofs::{
    circuit::{Layouter, Chip, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Constraints, Instance, Selector},
    poly::Rotation,
};

use std::marker::PhantomData;

#[derive(Debug, Clone)]
struct WalletChip<F: Field> {
    config: WalletConfig,
    _marker: PhantomData<F>,
}


impl<F: Field> WalletChip<F> {
    fn construct(config: <Self as Chip<F>>::Config) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 1],
        instance: Column<Instance>,
    ) -> WalletConfig {
        meta.enable_equality(instance);
        for column in &advice {
            meta.enable_equality(*column);   
        }
        let selector = meta.selector();

        meta.create_gate("wallet_address", |meta| {
            let s = meta.query_selector(selector);
            let ac = meta.query_advice(advice[0], Rotation::cur());
            Constraints::with_selector(s, vec![ac.clone() - ac])
        });

        WalletConfig {
            advice,
            selector,
            instance,
        }
    }
}

impl<F: Field> Chip<F> for WalletChip<F> {
    type Config = WalletConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

#[derive(Clone, Debug)]
pub struct WalletConfig {
    pub advice: [Column<Advice>; 1],
    pub instance: Column<Instance>,
    pub selector: Selector,
}

#[derive(Default, Clone)]
pub struct WalletCirciut<F: Field> {
    pub wallet_address: Value<F>,
    pub _marker: PhantomData<F>,
}

impl<F: Field> Circuit<F> for WalletCirciut<F> {
    type Config = WalletConfig;
    type FloorPlanner = SimpleFloorPlanner;


    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = [meta.advice_column()];
        let instance = meta.instance_column();
        
        WalletChip::configure(meta, advice, instance)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), halo2_proofs::plonk::Error> {
        let wallet_chip = WalletChip::<F>::construct(config);

        let out = layouter.assign_region(|| "confirm_wallet", 
            |mut region| {
                let advice = wallet_chip.config.advice;
                let s = wallet_chip.config.selector;

                s.enable(&mut region, 0)?;
                let wallet_address = region.assign_advice(
                    || "address", advice[0], 0, || self.wallet_address)?;

                Ok(wallet_address)
            },
        )?;
        layouter
            .namespace(|| "out")
            .constrain_instance(out.cell(), wallet_chip.config.instance, 0)
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::WalletCirciut;
    use hex;
    use halo2_curves::{
        serde::SerdeObject,
        bn256::Fr
    };
    use halo2_proofs::{dev::MockProver, circuit::Value};

    #[test]
    fn verify() {
        let k = 4;

        let address_str = "0x880262912356F79aAc79C00C1C9c0f6ce1BDD6ad".strip_prefix("0x").unwrap();
        let address = hex::decode(address_str).unwrap();
        let address = Fr::from_raw_bytes_unchecked(&[vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], address].concat());

        let circuit = WalletCirciut {
            wallet_address: Value::known(address), 
            _marker: PhantomData,
        };

        let public_inputs = vec![address];

        let prover = MockProver::run(k, &circuit, vec![public_inputs.clone()]).unwrap();
        assert_eq!(prover.verify(), Ok(()));
    }
}


fn main() {


}
