multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::contexts::base::State;

use crate::contexts::base::SwapTokensOrder;
use crate::contexts::base::StorageCache;

// use super::errors::*;

use crate::PairData;

use pair::ProxyTrait as _;
use pair::pair_actions::initial_liq::ProxyTrait as _;
use pair::pair_actions::common_result_types::AddLiquidityResultType;
use router::ProxyTrait as _;
use router::enable_swap_by_user::ProxyTrait as _;
use router::factory::PairFee;
use router::config::ProxyTrait as _;

pub const MAX_PERCENTAGE: u64 = 100_000;

// const LP_TXT: &[u8] = "LP".as_bytes();
// const JEET_TXT: &[u8] = "JEET".as_bytes();
// pub const MAX_FEE_PERCENTAGE: u64 = 5_000;

#[multiversx_sc::module]
pub trait ConfigModule:
    token_send::TokenSendModule
{   
    #[only_owner]
    #[endpoint]
    fn pause(&self) {
        self.state().set(State::Inactive);
    }

    #[only_owner]
    #[endpoint]
    fn resume(&self) {
        self.state().set(State::Active);
    }

    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[only_owner]
    #[endpoint(setStateActiveNoSwaps)]
    fn set_state_active_no_swaps(&self) {
        self.state().set(State::PartialActive);
    }
    #[only_owner]
    #[endpoint(changeFeesCollectorAddress)]
    fn change_fees_collector_address(&self, new_value: ManagedAddress) {
        self.fees_collector_address().set(new_value);
    }

    #[view(getFeesCollectorAddress)]
    #[storage_mapper("feesCollectorAddress")]
    fn fees_collector_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("tokenCreator")]
    fn token_creator(&self) -> SingleValueMapper<ManagedAddress>;


    #[only_owner]
    #[endpoint(setFeePercents)]
    fn set_fee_percent(&self, 
        total_fee_percent: u64) {
        self.total_fee_percent().set(total_fee_percent);
    }


    
    //FEES
    #[view(getOwnerFeePercent)]
    #[storage_mapper("total_fee_percent")]
    fn total_fee_percent(&self) -> SingleValueMapper<u64>;

    #[storage_mapper("reach_jeetdex_fee")]
    fn reach_jeetdex_fee(&self) -> SingleValueMapper<BigUint>;

    #[view(getFirstTokenId)]
    #[storage_mapper("first_token_id")]
    fn first_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getSecondTokenId)]
    #[storage_mapper("second_token_id")]
    fn second_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("ivl")]
    fn initial_virtual_liquidity(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("token_supply")]
    fn token_supply(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("oracle_address")]
    fn oracle_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("db_id")]
    fn db_id(&self) -> SingleValueMapper<ManagedBuffer>;

    #[storage_mapper("jeetdex_reach_pair")]
    fn jeetdex_reach_pair(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("jeetdex_router_sc_address")]
    fn jeetdex_router_sc_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(setJeetDexRouter)]
    fn set_jeetdex_router(&self, address: ManagedAddress) {
        self.jeetdex_router_sc_address().set(&address);
    }

    #[view(getMaxMarketCap)]
    #[storage_mapper("max_market_cap")]
    fn max_market_cap(&self) -> SingleValueMapper<BigUint>;

    // #[view(getDexTokenFee)]
    // #[storage_mapper("dex_token_fee")]
    // fn dex_token_fee(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("issue_token_cost")]
    fn issue_token_cost(&self) -> SingleValueMapper<BigUint>;

    #[storage_mapper("wegld_unwrap_sc")]
    fn wegld_unwrap_sc(&self) -> SingleValueMapper<ManagedAddress>;

    // #[storage_mapper("jeet_token_id")]
    // fn jeet_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getOwner)]
    #[storage_mapper("owner")]
    fn owner(&self) -> SingleValueMapper<ManagedAddress>;


    #[view(getReserve)]
    #[storage_mapper("reserve")]
    fn bonding_reserve(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getBondingData)]
    fn get_pair_data(&self) -> PairData<Self::Api> {
        PairData{
            first_token_id: self.first_token_id().get(),
            second_token_id: self.second_token_id().get(),
            first_token_reserve : self.bonding_reserve(&self.first_token_id().get()).get(),
            second_token_reserve : self.bonding_reserve(&self.second_token_id().get()).get(),
            owner_fee_percent: self.total_fee_percent().get(),
            market_cap: self.get_market_cap(),
            db_id: self.db_id().get(),
            state: self.state().get()
        }
    }

    #[view(getMarketCap)]
    fn get_market_cap(&self) -> BigUint{
        self.get_market_cap_internal(&self.bonding_reserve(&self.first_token_id().get()).get(), &self.bonding_reserve(&self.second_token_id().get()).get())
    }

    fn get_market_cap_internal(&self,
        first_token_reserve: &BigUint,
        second_token_reserve: &BigUint
    ) -> BigUint{
        let total_supply = self.token_supply().get();
        
        let token_price_egld = (second_token_reserve * &BigUint::from(10u32).pow(18u32)) /  first_token_reserve;
        let egld_price_in_usdc: BigUint = self.oracle_proxy(self.oracle_address().get()).get_amount_out_view(self.second_token_id().get(), BigUint::from(10u32).pow(18u32)).execute_on_dest_context_readonly();
        let token_price_usdc = (&token_price_egld * &egld_price_in_usdc);

        (&total_supply * &token_price_usdc) / BigUint::from(10u32).pow(36u32)


    }

    fn launch_token_on_jeetdex(&self, storage_cache: &mut StorageCache<Self>) {
        // get Jeet Router Pair creation fee

        let jeetdex_egld_fee: BigUint = self.jeetdex_router_proxy(self.jeetdex_router_sc_address().get())
            .new_pair_fee()
            .execute_on_dest_context_readonly();


        require!(self.bonding_reserve(&self.second_token_id().get()).get() 
            >  &self.issue_token_cost().get() + &jeetdex_egld_fee + &self.reach_jeetdex_fee().get(),
            "Fees greater than reserve");

        //We need to remove 0.05 + jeetdex egld fee from reserve and convert to EGLD to create LP token
        *storage_cache.get_mut_reserve_out(SwapTokensOrder::PoolOrder) -=
            &self.issue_token_cost().get() + &jeetdex_egld_fee + &self.reach_jeetdex_fee().get();

        let r:IgnoreValue = self.oracle_proxy(self.wegld_unwrap_sc().get())
            .unwrap_egld()
            .with_egld_or_single_esdt_transfer(EsdtTokenPayment::new(
                self.second_token_id().get(),
                0,
                self.issue_token_cost().get() + jeetdex_egld_fee.clone() + &self.reach_jeetdex_fee().get()
            ))
            .execute_on_dest_context();

        self.send().direct_egld(&self.fees_collector_address().get(), &self.reach_jeetdex_fee().get());

        //Create new pair
        
        require!(self.jeetdex_reach_pair().get() != ManagedAddress::zero(), "No pair exists");

        // let new_pair_address: ManagedAddress = self.jeetdex_router_proxy(self.jeetdex_router_sc_address().get())
        //     .create_pair_endpoint(self.first_token_id().get(),self.jeet_token_id().get(),OptionalValue::<PairFee>::None,OptionalValue::<PairFee>::None)
        //     .with_egld_transfer(jeetdex_egld_fee)
        //     .execute_on_dest_context();

        
        let egld_balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::egld(),0);
        
        require!(egld_balance >= self.issue_token_cost().get(), "Issue token cost does not match");
        
        if self.blockchain().get_gas_left() > 200000000 {
            // self.jeetdex_router_proxy(self.jeetdex_router_sc_address().get())
            // let r:IgnoreValue = 
            self.jeetdex_pair_proxy(self.jeetdex_reach_pair().get())
                .issue_lp_token_and_set_roles(
                    // new_pair_address,
                    self.get_display_name(&self.first_token_id().get()),
                    self.get_lp_token_ticker(&self.first_token_id().get())
                )
                .with_egld_transfer(self.issue_token_cost().get())
                .with_gas_limit(200000000)
                .transfer_execute();
        }else{
            sc_panic!("Not enough gas left {}", self.blockchain().get_gas_left());
        }

    }

    #[endpoint(addLiquidityAndEnableSwap)]
    fn continue_add_liquidity_and_enable_swap(&self, jeetdex_pair_address: ManagedAddress, lp_token_id: TokenIdentifier) {
        let caller = self.blockchain().get_caller();
        require!(caller == self.jeetdex_reach_pair().get(), "Not allowed");
        
        //Add Liquidity

        //SWAP WEGLD TO JEET
        let current_wegld_funds = self.bonding_reserve(&self.second_token_id().get()).get() - self.initial_virtual_liquidity().get();
        require!(current_wegld_funds == self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(self.second_token_id().get()),0),"Token reserves does not match");
        // let jeet_payment: EsdtTokenPayment = self.oracle_proxy(self.jeet_wegld_sc_address().get())
        //         .swap_tokens_fixed_input(self.jeet_token_id().get(),BigUint::from(1u32))
        //         .with_esdt_transfer(EsdtTokenPayment::new(self.second_token_id().get(), 0,current_funds ))
        //         .execute_on_dest_context();


        let mut payments = ManagedVec::new();
        payments.push(EsdtTokenPayment::new(self.first_token_id().get(), 0, self.bonding_reserve(&self.first_token_id().get()).get()));
        payments.push(EsdtTokenPayment::new(self.second_token_id().get(), 0, current_wegld_funds));

        // sc_panic!("hh");
        let mut r: AddLiquidityResultType<Self::Api>;
        let (r,back_transfers) = self.jeetdex_pair_proxy(jeetdex_pair_address.clone())
                .add_initial_liquidity()
                .with_multi_token_transfer(payments)
                .execute_on_dest_context_with_back_transfers::<AddLiquidityResultType<Self::Api>>();

        //Burn LP tokens
        for back_transfer in back_transfers.esdt_payments.iter(){
            if back_transfer.token_identifier == lp_token_id {
                self.send().esdt_local_burn(&back_transfer.token_identifier, back_transfer.token_nonce, &back_transfer.amount);
            }
        }

        //Enable Swaps on Pair
        let r:IgnoreValue = self.jeetdex_router_proxy(self.jeetdex_router_sc_address().get())
                .set_swap_enabled_by_user(jeetdex_pair_address)
                .execute_on_dest_context();
        
        self.state().set(State::Finished);
    }

    fn get_display_name(&self, token_id: &TokenIdentifier) -> ManagedBuffer{

        let mut display_name = ManagedBuffer::new();
        let jeetlp_text = ManagedBuffer::new_from_bytes("JEETLP".as_bytes());

        display_name.append(&self.get_token_ticker(token_id));
        display_name.append(&jeetlp_text);

        display_name
    }

    fn get_token_ticker(&self, token_id: &TokenIdentifier) -> ManagedBuffer {
        let token_id_managed_buffer = token_id.as_managed_buffer();
        let token_bytes = self.load_512_bytes(token_id_managed_buffer.clone());

        let mut slice = ManagedBuffer::new();

        for (i, &byte) in token_bytes.iter().enumerate() {
            if byte == "-".as_bytes()[0] || i >= token_id_managed_buffer.len() {
                slice = ManagedBuffer::new_from_bytes(&token_bytes[0..i]);
                break
            }
        }
        slice
    }

    fn get_lp_token_ticker(&self, token_id: &TokenIdentifier) -> ManagedBuffer{

        let token_ticker = self.get_token_ticker(token_id);

        let token_bytes = self.load_512_bytes(token_ticker.clone());

        let token_id_len = token_ticker.len();

        let substring_len = if token_id_len >= 6 {
            6
        }else{
            token_id_len
        };

        let mut token_ticker = ManagedBuffer::new();
        let substring_managed_buffer = ManagedBuffer::new_from_bytes(&token_bytes[0..substring_len - 1]);
        let jeet_text = ManagedBuffer::new_from_bytes("JEET".as_bytes());
        token_ticker.append(&substring_managed_buffer);
        token_ticker.append(&jeet_text);

        token_ticker

    }
    fn load_512_bytes(&self, text: ManagedBuffer) -> [u8; 512] {
        if (text.len() as usize) > 512 {
            sc_panic!("ManagedBuffer is too big");
        }

        let mut bytes: [u8; 512] = [0; 512];

        text.load_to_byte_array(&mut bytes);

        return bytes;
    }

    #[proxy]
    fn oracle_proxy(&self, to: ManagedAddress) -> oracle_proxy::Proxy<Self::Api>;
    #[proxy]
    fn jeetdex_pair_proxy(&self, to: ManagedAddress) -> pair::Proxy<Self::Api>;
    #[proxy]
    fn jeetdex_router_proxy(&self, to: ManagedAddress) -> router::Proxy<Self::Api>;


}


mod oracle_proxy {
    multiversx_sc::imports!();

    #[multiversx_sc::proxy]
   pub trait xExchangeOracleContract {
        #[view(getAmountOut)]
        fn get_amount_out_view(&self, token_in: TokenIdentifier, amount_in: BigUint);

        #[payable("*")]
        #[endpoint(swapTokensFixedInput)]
        fn swap_tokens_fixed_input(&self,token_out: TokenIdentifier,amount_out_min: BigUint);

        #[payable("*")]
        #[endpoint(unwrapEgld)]
        fn unwrap_egld(&self);

        #[payable("EGLD")]
        #[endpoint(wrapEgld)]
        fn wrap_egld(&self);
    }
}