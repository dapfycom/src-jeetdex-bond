use crate::contexts::base::StorageCache;
use crate::contexts::swap::SwapContext;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode)]
pub struct SwapEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    token_id_in: TokenIdentifier<M>,
    token_amount_in: BigUint<M>,
    token_id_out: TokenIdentifier<M>,
    token_amount_out: BigUint<M>,
    fee_amount: BigUint<M>,
    token_in_reserve: BigUint<M>,
    token_out_reserve: BigUint<M>,
    block: u64,
    epoch: u64,
    timestamp: u64,
}





#[multiversx_sc::module]
pub trait EventsModule:
    crate::config::ConfigModule
    + token_send::TokenSendModule
{
    fn emit_swap_event(&self, storage_cache: &StorageCache<Self>, context: SwapContext<Self::Api>) {
        let epoch = self.blockchain().get_block_epoch();
        let caller = self.blockchain().get_caller();
        self.swap_event(
            &context.input_token_id.clone(),
            &context.output_token_id.clone(),
            &caller,
            epoch,
            &SwapEvent {
                caller: caller.clone(),
                token_id_in: context.input_token_id,
                token_amount_in: context.final_input_amount,
                token_id_out: context.output_token_id,
                token_amount_out: context.final_output_amount,
                fee_amount: context.fee_amount,
                token_in_reserve: storage_cache
                    .get_reserve_in(context.swap_tokens_order)
                    .clone(),
                token_out_reserve: storage_cache
                    .get_reserve_out(context.swap_tokens_order)
                    .clone(),
                block: self.blockchain().get_block_nonce(),
                epoch,
                timestamp: self.blockchain().get_block_timestamp(),
            },
        )
    }

    #[event("swap")]
    fn swap_event(
        &self,
        #[indexed] token_in: &TokenIdentifier,
        #[indexed] token_out: &TokenIdentifier,
        #[indexed] caller: &ManagedAddress,
        #[indexed] epoch: u64,
        swap_event: &SwapEvent<Self::Api>,
    );

}
