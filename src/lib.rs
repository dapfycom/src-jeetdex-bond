#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

mod amm
;
pub mod config;
pub mod contexts;
pub mod errors;
mod events;
pub mod fee;
pub mod pair_actions;

use crate::errors::*;

use router::ProxyTrait as _;
use router::factory::PairFee;

use contexts::base::*;
use pair_actions::common_result_types::SwapTokensFixedInputResultType;

const DEFAULT_OWNER_FEE_PERCENT: u64 = 1000;

#[multiversx_sc::contract]
pub trait Pair<ContractReader>:
    amm::AmmModule
    + fee::FeeModule
    + config::ConfigModule
    + token_send::TokenSendModule
    + events::EventsModule
    + contexts::output_builder::OutputBuilderModule
    + pair_actions::swap::SwapModule
    + pair_actions::views::ViewsModule
    + pair_actions::common_methods::CommonMethodsModule
{
    #[init]
    fn init(
        &self,
        allowed_token: TokenIdentifier,
        fees_collector: ManagedAddress,
        initial_virtual_liquidity: BigUint,
        // dex_token_fee: BigUint,
        oracle_address: ManagedAddress,
        max_market_cap: BigUint,
        jeetdex_router_sc_address: ManagedAddress,
        issue_token_cost: BigUint,
        wegld_unwrap_sc: ManagedAddress,
        reach_jeetdex_fee: BigUint,
        db_id: ManagedBuffer
        
    ) {
        require!(allowed_token.is_valid_esdt_identifier(), ERROR_NOT_AN_ESDT);


        self.set_fee_percent(DEFAULT_OWNER_FEE_PERCENT);
        self.state().set(State::Inactive);

        self.fees_collector_address().set(&fees_collector);
        self.second_token_id().set_if_empty(&allowed_token);
        self.initial_virtual_liquidity().set_if_empty(&initial_virtual_liquidity);
        // self.dex_token_fee().set_if_empty(dex_token_fee);
        self.oracle_address().set_if_empty(oracle_address);
        self.max_market_cap().set_if_empty(max_market_cap);
        self.jeetdex_router_sc_address().set_if_empty(jeetdex_router_sc_address);
        self.issue_token_cost().set_if_empty(issue_token_cost);
        self.wegld_unwrap_sc().set_if_empty(wegld_unwrap_sc);
        self.owner().set(self.blockchain().get_caller());
        self.reach_jeetdex_fee().set_if_empty(reach_jeetdex_fee);
        self.db_id().set_if_empty(db_id);

    }

    #[endpoint]
    fn upgrade(&self,
        allowed_token: TokenIdentifier,
        fees_collector: ManagedAddress,
        initial_virtual_liquidity: BigUint,
        // dex_token_fee: BigUint,
        oracle_address: ManagedAddress,
        max_market_cap: BigUint,
        jeetdex_router_sc_address: ManagedAddress,
        // jeet_wegld_sc_address: ManagedAddress,
        issue_token_cost: BigUint,
        wegld_unwrap_sc: ManagedAddress,
        reach_jeetdex_fee: BigUint,
        db_id: ManagedBuffer
    ) {}

    #[payable("*")]
    #[endpoint(setTokenIdentifier)]
    fn set_token_identifier(&self, token_creator_buy: bool, token_creator: ManagedAddress) {
        require!(self.blockchain().get_caller() == self.owner().get(),"Only owner endpoint");
        
        require!(self.first_token_id().is_empty(),"Tokens already set");

        require!(self.state().get() == State::Inactive, "Wrong State");

        let payment = self.call_value().single_esdt();
        
        self.first_token_id().set(&payment.token_identifier);

        let mut storage_cache = StorageCache::new(self);

        self.token_supply().set(payment.amount.clone());

        storage_cache.first_token_reserve += payment.amount;
        storage_cache.second_token_reserve += self.initial_virtual_liquidity().get();

        self.token_creator().set(token_creator);


        //Create new pair
        let new_pair_address: ManagedAddress = self.jdex_router_proxy(self.jeetdex_router_sc_address().get())
        // .create_pair_endpoint(first_token_id, second_token_id, opt_buy_fee_percents, opt_sell_fee_percents)
            .create_pair_endpoint(self.first_token_id().get(),self.second_token_id().get(),OptionalValue::<PairFee>::None,OptionalValue::<PairFee>::None)
            .execute_on_dest_context();

        self.jeetdex_reach_pair().set(new_pair_address);

        if token_creator_buy == false {
            self.state().set(State::Active);
        }
                

    }   
    #[proxy]
    fn jdex_router_proxy(&self, to: ManagedAddress) -> router::Proxy<Self::Api>;
}
