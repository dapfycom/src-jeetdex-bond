use crate::{
    contexts::swap::SwapContext,StorageCache, ERROR_INVALID_ARGS, ERROR_K_INVARIANT_FAILED, ERROR_NOT_ENOUGH_RESERVE, ERROR_SLIPPAGE_EXCEEDED, ERROR_SWAP_NOT_ENABLED, ERROR_ZERO_AMOUNT
};

use crate::contexts::base::State;
use super::common_result_types::SwapTokensFixedInputResultType;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy)]
pub enum SwapType {
    FixedInput,
    FixedOutput,
}

#[multiversx_sc::module]
pub trait SwapModule:
    crate::amm::AmmModule
    + crate::contexts::output_builder::OutputBuilderModule
    + crate::events::EventsModule
    + crate::fee::FeeModule
    + crate::config::ConfigModule
    + token_send::TokenSendModule
    + super::common_methods::CommonMethodsModule
{   
    #[payable("*")]
    #[endpoint(initialSwap)]
    fn initial_swap_tokens_fixed_input(
        &self,
        token_out: TokenIdentifier,
        amount_out_min: BigUint,
    ) -> SwapTokensFixedInputResultType<Self::Api> {
        require!(self.blockchain().get_caller() == self.token_creator().get(), "You are not allowed to buy");
        require!(self.state().get() != State::Active,"Bonding is already active");
        self.state().set(State::Active);

        self.swap_tokens_fixed_input_internal(token_out, amount_out_min,self.call_value().single_esdt())
    }

    #[payable("*")]
    #[endpoint(swap)]
    fn swap_tokens_fixed_input(
        &self,
        token_out: TokenIdentifier,
        amount_out_min: BigUint,
    ) -> SwapTokensFixedInputResultType<Self::Api> {

        
        require!(amount_out_min > 0, ERROR_INVALID_ARGS);

        let mut storage_cache = StorageCache::new(self);
        let payment = self.call_value().single_esdt();
        let swap_tokens_order =
            storage_cache.get_swap_tokens_order(&payment.token_identifier, &token_out);

        require!(
            self.can_swap(storage_cache.contract_state),
            ERROR_SWAP_NOT_ENABLED
        );

        let reserve_out = storage_cache.get_mut_reserve_out(swap_tokens_order);
        require!(*reserve_out > amount_out_min, ERROR_NOT_ENOUGH_RESERVE);
        
        let initial_k = self.calculate_k_constant(
            &storage_cache.first_token_reserve,
            &storage_cache.second_token_reserve,
        );

        // sc_panic!("{} {} {}",&storage_cache.first_token_reserve,&storage_cache.second_token_reserve,initial_k);

        let mut swap_context = SwapContext::new(
            payment.token_identifier,
            payment.amount,
            token_out,
            amount_out_min,
            swap_tokens_order,
        );

        self.perform_swap_fixed_input(&mut swap_context, &mut storage_cache);

        let new_k = self.calculate_k_constant(
            &storage_cache.first_token_reserve,
            &storage_cache.second_token_reserve,
        );
        // sc_panic!("{} {} {} {}",&storage_cache.first_token_reserve,&storage_cache.second_token_reserve,initial_k,new_k);
        require!(initial_k <= new_k, ERROR_K_INVARIANT_FAILED);

        if swap_context.fee_amount > 0 {
            self.send_fee(
                &swap_context.input_token_id,
                &swap_context.fee_amount,
            );
        }

        let caller = self.blockchain().get_caller();
        let output_payments = self.build_swap_output_payments(&swap_context);

        require!(
            output_payments.get(0).amount >= swap_context.output_token_amount,
            ERROR_SLIPPAGE_EXCEEDED
        );
        // sc_panic!("AAA {} {} {} {}",output_payments.get(0).token_identifier,output_payments.get(0).amount,output_payments.get(1).token_identifier,output_payments.get(1).amount);
        self.send_multiple_tokens_if_not_zero(&caller, &output_payments);

        self.emit_swap_event(&storage_cache, swap_context);

        // sc_panic!("AAA {} {} {} {}",
        //     storage_cache.get_reserve_in(swap_tokens_order),
        //     storage_cache.get_reserve_out(swap_tokens_order),
        //     storage_cache.first_token_reserve,
        //     storage_cache.second_token_reserve,
        // );

        let market_cap = self.get_market_cap_internal(&storage_cache.first_token_reserve,&storage_cache.second_token_reserve);
        if market_cap >= self.max_market_cap().get(){
            // Let's go to JeetDex
            self.launch_token_on_jeetdex(&mut storage_cache);
        }

        self.build_swap_fixed_input_results(output_payments)
    }

    fn swap_tokens_fixed_input_internal(
        &self,
        token_out: TokenIdentifier,
        amount_out_min: BigUint,
        internal_payment: EsdtTokenPayment
    ) -> SwapTokensFixedInputResultType<Self::Api> {
        require!(amount_out_min > 0, ERROR_INVALID_ARGS);

        let mut storage_cache = StorageCache::new(self);
        let payment = internal_payment;
        let swap_tokens_order =
            storage_cache.get_swap_tokens_order(&payment.token_identifier, &token_out);

        require!(
            self.can_swap(storage_cache.contract_state),
            ERROR_SWAP_NOT_ENABLED
        );

        let reserve_out = storage_cache.get_mut_reserve_out(swap_tokens_order);
        require!(*reserve_out > amount_out_min, ERROR_NOT_ENOUGH_RESERVE);
        
        let initial_k = self.calculate_k_constant(
            &storage_cache.first_token_reserve,
            &storage_cache.second_token_reserve,
        );

        // sc_panic!("{} {} {}",&storage_cache.first_token_reserve,&storage_cache.second_token_reserve,initial_k);

        let mut swap_context = SwapContext::new(
            payment.token_identifier,
            payment.amount,
            token_out,
            amount_out_min,
            swap_tokens_order,
        );

        self.perform_swap_fixed_input(&mut swap_context, &mut storage_cache);

        let new_k = self.calculate_k_constant(
            &storage_cache.first_token_reserve,
            &storage_cache.second_token_reserve,
        );
        // sc_panic!("{} {} {} {}",&storage_cache.first_token_reserve,&storage_cache.second_token_reserve,initial_k,new_k);
        require!(initial_k <= new_k, ERROR_K_INVARIANT_FAILED);

        if swap_context.fee_amount > 0 {
            self.send_fee(
                &swap_context.input_token_id,
                &swap_context.fee_amount,
            );
        }

        let caller = self.blockchain().get_caller();
        let output_payments = self.build_swap_output_payments(&swap_context);

        require!(
            output_payments.get(0).amount >= swap_context.output_token_amount,
            ERROR_SLIPPAGE_EXCEEDED
        );

        self.send_multiple_tokens_if_not_zero(&caller, &output_payments);

        self.emit_swap_event(&storage_cache, swap_context);

        

        let market_cap = self.get_market_cap_internal(&storage_cache.first_token_reserve,&storage_cache.second_token_reserve);
        if market_cap >= self.max_market_cap().get(){
            // Let's go to JeetDex
            self.launch_token_on_jeetdex(&mut storage_cache);
        }

        self.build_swap_fixed_input_results(output_payments)
    }


    fn perform_swap_fixed_input(
        &self,
        context: &mut SwapContext<Self::Api>,
        storage_cache: &mut StorageCache<Self>
    ) {
        context.final_input_amount = context.input_token_amount.clone();

        let reserve_in = storage_cache.get_reserve_in(context.swap_tokens_order);
        let reserve_out = storage_cache.get_reserve_out(context.swap_tokens_order);
        
        let amount_out_optimal =
            self.get_amount_out(&context.input_token_amount, reserve_in, reserve_out);
        require!(
            amount_out_optimal >= context.output_token_amount,
            ERROR_SLIPPAGE_EXCEEDED
        );
        require!(*reserve_out > amount_out_optimal, ERROR_NOT_ENOUGH_RESERVE);
        require!(amount_out_optimal != 0u64, ERROR_ZERO_AMOUNT);

        context.final_output_amount = amount_out_optimal;

        let mut amount_in_after_fee = context.input_token_amount.clone();
        if self.is_fee_enabled() {
            let fee_amount = self.get_total_fee_from_input(&amount_in_after_fee);
            amount_in_after_fee -= &fee_amount;

            context.fee_amount = fee_amount;
        }

        *storage_cache.get_mut_reserve_in(context.swap_tokens_order) += amount_in_after_fee;
        *storage_cache.get_mut_reserve_out(context.swap_tokens_order) -=
            &context.final_output_amount;
    }


}
