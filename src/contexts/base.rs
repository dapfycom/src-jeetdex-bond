multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode,TypeAbi, TopEncode, TopDecode, PartialEq, Copy, Clone, Debug,ManagedVecItem)]
pub enum State {
    Inactive,
    Active,
    PartialActive,
    PendingAddLiquidity,
    Finished,
}

#[derive(PartialEq, Copy, Clone)]
pub enum SwapTokensOrder {
    PoolOrder,
    ReverseOrder,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, TypeAbi, Clone,ManagedVecItem)]
pub struct PairData<M: ManagedTypeApi> {
    pub first_token_id: TokenIdentifier<M>,
    pub second_token_id: TokenIdentifier<M>,
    pub first_token_reserve: BigUint<M>,
    pub second_token_reserve: BigUint<M>,
    pub owner_fee_percent: u64,
    pub market_cap: BigUint<M>,
    pub db_id: ManagedBuffer<M>,
    pub state: State
}

pub struct StorageCache<'a, C>
where
    C: crate::config::ConfigModule,
{
    sc_ref: &'a C,
    pub contract_state: State,
    pub first_token_id: TokenIdentifier<C::Api>,
    pub second_token_id: TokenIdentifier<C::Api>,
    pub first_token_reserve: BigUint<C::Api>,
    pub second_token_reserve: BigUint<C::Api>,
}

impl<'a, C> StorageCache<'a, C>
where
    C: crate::config::ConfigModule,
{
    pub fn new(sc_ref: &'a C) -> Self {
        let first_token_id = sc_ref.first_token_id().get();
        let second_token_id = sc_ref.second_token_id().get();
        let first_token_reserve = sc_ref.bonding_reserve(&first_token_id).get();
        let second_token_reserve = sc_ref.bonding_reserve(&second_token_id).get();

        StorageCache {
            contract_state: sc_ref.state().get(),
            first_token_id,
            second_token_id,
            first_token_reserve,
            second_token_reserve,
            sc_ref,
        }
    }

    pub fn get_swap_tokens_order(
        &self,
        first_token_id: &TokenIdentifier<C::Api>,
        second_token_id: &TokenIdentifier<C::Api>,
    ) -> SwapTokensOrder {
        if first_token_id == &self.first_token_id && second_token_id == &self.second_token_id {
            SwapTokensOrder::PoolOrder
        } else if first_token_id == &self.second_token_id && second_token_id == &self.first_token_id
        {
            SwapTokensOrder::ReverseOrder
        } else {
            multiversx_sc::contract_base::ErrorHelper::<C::Api>::signal_error_with_message(
                &b"Invalid tokens"[..],
            );
        }
    }

    pub fn get_reserve_in(&self, swap_tokens_order: SwapTokensOrder) -> &BigUint<C::Api> {
        match swap_tokens_order {
            SwapTokensOrder::PoolOrder => &self.first_token_reserve,
            SwapTokensOrder::ReverseOrder => &self.second_token_reserve,
        }
    }

    pub fn get_reserve_out(&self, swap_tokens_order: SwapTokensOrder) -> &BigUint<C::Api> {
        match swap_tokens_order {
            SwapTokensOrder::PoolOrder => &self.second_token_reserve,
            SwapTokensOrder::ReverseOrder => &self.first_token_reserve,
        }
    }

    pub fn get_mut_reserve_in(
        &mut self,
        swap_tokens_order: SwapTokensOrder,
    ) -> &mut BigUint<C::Api> {
        match swap_tokens_order {
            SwapTokensOrder::PoolOrder => &mut self.first_token_reserve,
            SwapTokensOrder::ReverseOrder => &mut self.second_token_reserve,
        }
    }

    pub fn get_mut_reserve_out(
        &mut self,
        swap_tokens_order: SwapTokensOrder,
    ) -> &mut BigUint<C::Api> {
        match swap_tokens_order {
            SwapTokensOrder::PoolOrder => &mut self.second_token_reserve,
            SwapTokensOrder::ReverseOrder => &mut self.first_token_reserve,
        }
    }
}

impl<'a, C> Drop for StorageCache<'a, C>
where
    C: crate::config::ConfigModule,
{
    fn drop(&mut self) {
        // commit changes to storage for the mutable fields
        self.sc_ref
            .bonding_reserve(&self.first_token_id)
            .set(&self.first_token_reserve);

        self.sc_ref
            .bonding_reserve(&self.second_token_id)
            .set(&self.second_token_reserve);

    }
}
