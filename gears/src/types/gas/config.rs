#![allow(dead_code)] // TODO: Remove

use super::FiniteGas;

pub struct GasConfig {
    has_cost: FiniteGas,
    delete_cost: FiniteGas,
    read_cost_flat: FiniteGas,
    read_cost_per_byte: FiniteGas,
    write_cost_flat: FiniteGas,
    write_cost_per_byte: FiniteGas,
    iter_next_cost_flat: FiniteGas,
}

impl GasConfig {
    pub fn default_kv() -> Self {
        Self {
            has_cost: FiniteGas(1000),
            delete_cost: FiniteGas(1000),
            read_cost_flat: FiniteGas(1000),
            read_cost_per_byte: FiniteGas(3),
            write_cost_flat: FiniteGas(2000),
            write_cost_per_byte: FiniteGas(30),
            iter_next_cost_flat: FiniteGas(30),
        }
    }

    pub fn default_transient() -> Self {
        Self {
            has_cost: FiniteGas(100),
            delete_cost: FiniteGas(100),
            read_cost_flat: FiniteGas(100),
            read_cost_per_byte: FiniteGas(0),
            write_cost_flat: FiniteGas(200),
            write_cost_per_byte: FiniteGas(3),
            iter_next_cost_flat: FiniteGas(3),
        }
    }
}
