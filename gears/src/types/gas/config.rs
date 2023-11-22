#![allow(dead_code)] // TODO: Remove

use super::gas_meter::Gas;

pub struct GasConfig {
    has_cost: Gas,
    delete_cost: Gas,
    read_cost_flat: Gas,
    read_cost_per_byte: Gas,
    write_cost_flat: Gas,
    write_cost_per_byte: Gas,
    iter_next_cost_flat: Gas,
}

impl GasConfig {
    pub fn default_kv() -> Self {
        Self {
            has_cost: Gas(1000),
            delete_cost: Gas(1000),
            read_cost_flat: Gas(1000),
            read_cost_per_byte: Gas(3),
            write_cost_flat: Gas(2000),
            write_cost_per_byte: Gas(30),
            iter_next_cost_flat: Gas(30),
        }
    }

    pub fn default_transient() -> Self {
        Self {
            has_cost: Gas(100),
            delete_cost: Gas(100),
            read_cost_flat: Gas(100),
            read_cost_per_byte: Gas(0),
            write_cost_flat: Gas(200),
            write_cost_per_byte: Gas(3),
            iter_next_cost_flat: Gas(3),
        }
    }
}
