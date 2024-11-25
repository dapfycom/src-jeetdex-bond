multiversx_sc::imports!();

use crate::SwapTokensFixedInputResultType;

use super::swap::SwapContext;

#[multiversx_sc::module]
pub trait OutputBuilderModule:
    crate::config::ConfigModule
    + token_send::TokenSendModule
{
    fn build_swap_output_payments(
        &self,
        swap_context: &SwapContext<Self::Api>,
    ) -> ManagedVec<EsdtTokenPayment<Self::Api>> {
        let mut payments = ManagedVec::new();

        
        payments.push(EsdtTokenPayment::new(
            swap_context.output_token_id.clone(),
            0,
            swap_context.final_output_amount.clone(),
        ));
        

        let extra_amount = &swap_context.input_token_amount - &swap_context.final_input_amount;
        payments.push(EsdtTokenPayment::new(
            swap_context.input_token_id.clone(),
            0,
            extra_amount,
        ));

        payments
    }

    #[inline]
    fn build_swap_fixed_input_results(
        &self,
        output_payments: ManagedVec<EsdtTokenPayment<Self::Api>>,
    ) -> SwapTokensFixedInputResultType<Self::Api> {
        output_payments.get(0)
    }

}
