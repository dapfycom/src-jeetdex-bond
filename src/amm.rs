multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::config::MAX_PERCENTAGE;

use super::config;

#[multiversx_sc::module]
pub trait AmmModule:
    config::ConfigModule
    + token_send::TokenSendModule
{
    fn calculate_k_constant(
        &self,
        first_token_amount: &BigUint,
        second_token_amount: &BigUint,
    ) -> BigUint {
        first_token_amount * second_token_amount
    }

    fn quote(
        &self,
        first_token_amount: &BigUint,
        first_token_reserve: &BigUint,
        second_token_reserve: &BigUint,
    ) -> BigUint {
        &(first_token_amount * second_token_reserve) / first_token_reserve
    }

    fn get_amount_out(
        &self,
        amount_in: &BigUint,
        reserve_in: &BigUint,
        reserve_out: &BigUint,
    ) -> BigUint {

        let amount_in_with_fee = amount_in * (MAX_PERCENTAGE - self.total_fee_percent().get());
        let numerator = &amount_in_with_fee * reserve_out;
        let denominator = (reserve_in * MAX_PERCENTAGE) + amount_in_with_fee;

        numerator / denominator
    }

    fn get_total_fee_from_input(&self, amount_in: &BigUint) -> BigUint {
        amount_in * self.total_fee_percent().get() / MAX_PERCENTAGE
    }

}
