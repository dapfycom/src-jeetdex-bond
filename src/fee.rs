multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use super::amm;
use super::config;
// use super::errors::*;
// use super::liquidity_pool;
// use crate::config::MAX_PERCENTAGE;
// use crate::contexts::base::StorageCache;
// use crate::contexts::base::SwapTokensOrder;

// use common_structs::TokenPair;


#[multiversx_sc::module]
pub trait FeeModule:
    config::ConfigModule
    + amm::AmmModule
    + token_send::TokenSendModule
{
    fn is_fee_enabled(&self) -> bool {
        !self.fees_collector_address().is_empty()
    }

    fn send_fee(
        &self,
        fee_token: &TokenIdentifier,
        fee_amount: &BigUint,
    ) {
        if fee_amount == &0u64 {
            return;
        }

        let fees_collector_configured = !self.fees_collector_address().is_empty();
        if fees_collector_configured && fee_amount > &BigUint::zero(){
            self.send_fees_collector_cut(fee_token.clone(), fee_amount.clone());
        }
        
    }

    fn send_fees_collector_cut(&self, token: TokenIdentifier, cut_amount: BigUint) {
        self.send_tokens_non_zero(&self.fees_collector_address().get(), &token, 0, &cut_amount);
    }


    #[inline]
    fn burn(&self, token: &TokenIdentifier, amount: &BigUint) {
        if amount > &0 {
            self.send().esdt_local_burn(token, 0, amount);
        }
    }



}
