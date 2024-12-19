use stylus_sdk::types::{Address, U256};
use stylus_sdk::erc20::ERC20;

use stylus_sdk::prelude::*;

pub struct TokenSale {
    token: ERC20,
    owner: Address,
    price_per_token: U256,
    tokens_sold: U256,
}

impl TokenSale {
    pub fn new(token_address: Address, price_per_token: U256) -> Self {
        let token = ERC20::new(token_address);
        let owner = stylus_sdk::env::caller();
        TokenSale {
            token,
            owner,
            price_per_token,
            tokens_sold: U256::zero(),
        }
    }

    pub fn buy_tokens(&mut self, amount: U256) {
        let caller = stylus_sdk::env::caller();
        let value = stylus_sdk::env::value();

        let total_cost = self.price_per_token * amount;
        assert!(value >= total_cost, "Insufficient funds sent");

        let token_balance = self.token.balance_of(self.owner);
        assert!(token_balance >= amount, "Not enough tokens available for sale");

        self.token.transfer_from(self.owner, caller, amount);
        self.tokens_sold += amount;

        // Refund any excess funds sent
        if value > total_cost {
            let refund = value - total_cost;
            stylus_sdk::env::transfer(caller, refund);
        }
    }

    pub fn withdraw(&self) {
        let caller = stylus_sdk::env::caller();
        assert!(caller == self.owner, "Only the owner can withdraw funds");

        let balance = stylus_sdk::env::balance();
        stylus_sdk::env::transfer(self.owner, balance);
    }
}