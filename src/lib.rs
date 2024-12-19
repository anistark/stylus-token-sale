#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// mod erc20; // Not Needed anymore
mod errors;

use stylus_sdk::prelude::sol_interface;
use alloc::vec::Vec;
use errors::{EndtimeInPast, NotAdmin, SaleEnded, TokenSaleErrors};
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    block, console, contract, msg,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU256},
};

// ERC20 Interface
sol_interface! {
  interface IERC20 {
    function transfer(address to, uint256 value) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256);
    function transferFrom(address from, address to, uint256 value) external returns (bool);
  }
}

// TokenSale contract entrypoint
#[storage]
#[entrypoint]
pub struct TokenSale {
    is_initialised: StorageBool,
    admin: StorageAddress,
    token: StorageAddress,
    total_supply: StorageU256,
    tokens_sold: StorageU256,
    token_price: StorageU256,
    sale_end: StorageU256,
    collected_amount: StorageMap<Address, StorageU256>,
}

// TokenSale contract public implementation
#[public]
impl TokenSale {

    // Initialise TokenSale
    pub fn initialise(
        &mut self,
        admin: Address,
        token: Address,
        total_supply: U256,
        sale_end: U256,
        token_price: U256,
        supported_tokens: Vec<Address>,
    ) -> Result<(), TokenSaleErrors> {
        if sale_end < U256::from(block::timestamp()) {
            return Err(TokenSaleErrors::EndtimeInPast(EndtimeInPast {}));
        }

        if supported_tokens.len() > 0 {
            for s_token in supported_tokens {
                self.collected_amount.insert(s_token, U256::ZERO);
            }
        }

        self.transfer_token_from(token, total_supply, admin, contract::address());
        self.is_initialised.set(true);
        self.admin.set(admin);
        self.token.set(token);
        self.tokens_sold.set(U256::ZERO);
        self.total_supply.set(total_supply);
        self.sale_end.set(sale_end);
        self.token_price.set(token_price);

        Ok(())
    }

    // Purchase Token
    pub fn buy_token(
        &mut self,
        amount: U256,
        token_in: Address,
    ) -> Result<(), TokenSaleErrors> {
        if self.sale_end.get() < U256::from(block::timestamp()) {
            return Err(TokenSaleErrors::SaleEnded(SaleEnded {}));
        }

        self.tokens_sold.set(self.tokens_sold.get() + self.token_price.get());

        self.transfer_token_from(token_in, amount, msg::sender(), contract::address());

        self.transfer_token(self.token.get(), self.token_price.get(), msg::sender());

        self.set_collected_amount(token_in, self.collected_amount.get(token_in) + amount);

        Ok(())
    }

    // Withdraw funds
    pub fn withdraw(&mut self, token_addr: Address) -> Result<(), TokenSaleErrors> {
        if msg::sender() != self.admin.get() {
            return Err(TokenSaleErrors::NotAdmin(NotAdmin {}));
        }

        self.transfer_token(
            token_addr,
            self.collected_amount.get(token_addr),
            msg::sender(),
        );

        self.set_collected_amount(token_addr, U256::from(0));

        Ok(())
    }

    pub fn is_initialised(&self) -> bool {
        self.is_initialised.get()
    }
}

// TokenSale contract implementation
impl TokenSale {

    // Transfer token from caller address to another
    fn transfer_token(&mut self, token_addr: Address, amount: U256, to: Address) {
        let token = IERC20::new(token_addr);
        let _ = token.transfer(&mut *self, to, amount).unwrap();
    }

    // Transfer token from one address to another
    fn transfer_token_from(
        &mut self,
        token_addr: Address,
        amount: U256,
        from: Address,
        to: Address,
    ) {
        let token = IERC20::new(token_addr);
        let _ = token.transfer_from(&mut *self, from, to, amount).unwrap();
    }

    // Set collected amount for a token
    fn set_collected_amount(&mut self, token_addr: Address, new_amount: U256) {
        let mut amount_setter = self.collected_amount.setter(token_addr);
        amount_setter.set(new_amount);
    }
}
