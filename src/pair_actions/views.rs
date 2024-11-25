use crate::{ ERROR_NOT_ENOUGH_RESERVE, ERROR_UNKNOWN_TOKEN, ERROR_ZERO_AMOUNT};

// use crate::contexts::*;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ViewsModule:
    crate::amm::AmmModule
    + crate::contexts::output_builder::OutputBuilderModule
    + crate::events::EventsModule
    + crate::fee::FeeModule
    + crate::config::ConfigModule
    + token_send::TokenSendModule
    + super::common_methods::CommonMethodsModule
{

    #[view(getAmountOut)]
    fn get_amount_out_view(&self, token_in: TokenIdentifier, amount_in: BigUint) -> BigUint {
        require!(amount_in > 0u64, ERROR_ZERO_AMOUNT);

        let first_token_id = self.first_token_id().get();
        let second_token_id = self.second_token_id().get();
        let first_token_reserve = self.bonding_reserve(&first_token_id).get();
        let second_token_reserve = self.bonding_reserve(&second_token_id).get();

        if token_in == first_token_id {
            require!(second_token_reserve > 0u64, ERROR_NOT_ENOUGH_RESERVE);
            let amount_out =
                self.get_amount_out(&amount_in, &first_token_reserve, &second_token_reserve);
            require!(second_token_reserve > amount_out, ERROR_NOT_ENOUGH_RESERVE);
            amount_out
        } else if token_in == second_token_id {
            require!(first_token_reserve > 0u64, ERROR_NOT_ENOUGH_RESERVE);
            let amount_out =
                self.get_amount_out(&amount_in, &second_token_reserve, &first_token_reserve);
            require!(first_token_reserve > amount_out, ERROR_NOT_ENOUGH_RESERVE);
            amount_out
        } else {
            sc_panic!(ERROR_UNKNOWN_TOKEN);
        }
    }
    
}
