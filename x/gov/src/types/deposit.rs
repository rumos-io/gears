use gears::types::base::coin::Coin;

pub struct Deposit {
    pub proposal_id: u64,
    pub depositor: String,
    pub amount: Vec<Coin>,
}
