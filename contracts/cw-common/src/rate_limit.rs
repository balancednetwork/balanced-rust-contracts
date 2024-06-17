use cosmwasm_schema::cw_serde;
use cosmwasm_std::{DepsMut, Env, StdError};
use cw_xcall_multi::error::ContractError;

use crate::helpers::{balance_of, bank_balance_of};
#[cw_serde]
pub struct RateLimit {
    pub period: u64,
    pub percentage: u32,
    pub last_update: u64,
    pub current_limit: u128,
}

pub const POINTS: u128 = 10000;
pub trait RateLimited {
    fn verify_withdraw(
        &self,
        deps: &DepsMut,
        env: Env,
        token: String,
        amount: u128,
        is_denom: bool,
    ) -> Result<RateLimit, ContractError>;
}

impl RateLimited for RateLimit {
    fn verify_withdraw(
        &self,
        deps: &DepsMut,
        env: Env,
        token: String,
        amount: u128,
        is_denom: bool,
    ) -> Result<RateLimit, ContractError> {
        if self.period == 0 {
            return Ok(self.clone());
        }

        let balance = match is_denom {
            true => bank_balance_of(&deps.as_ref(), token, env.contract.address.to_string())?,
            false => balance_of(&deps.as_ref(), token, env.contract.address.to_string())?,
        };

        let current_time = env.block.time.seconds();
        let max_limit = balance
            .checked_mul(self.percentage.into())
            .unwrap()
            .checked_div(POINTS)
            .unwrap();

        if self.current_limit == 0 {
            return Ok(RateLimit {
                percentage: self.percentage,
                period: self.period,
                last_update: current_time,
                current_limit: max_limit,
            });
        }

        // The maximum amount that can be withdraw in one period
        let max_withdraw = balance.checked_sub(max_limit).unwrap();
        let time_diff = current_time
            .checked_sub(self.last_update)
            .unwrap()
            .min(self.period);

        // The amount that should be added as available
        let added_allowed_withdrawal = max_withdraw
            .checked_mul(time_diff.into())
            .unwrap()
            .checked_div(self.period.into())
            .unwrap();

        let mut calculated_limit = self.current_limit;

        if self.current_limit > added_allowed_withdrawal {
            calculated_limit = self
                .current_limit
                .checked_sub(added_allowed_withdrawal)
                .unwrap();
        }

        // If the balance is below the limit then set limt to current balance (no withdraws are possible)
        // If limit goes below what the protected percentage is set it to the maxLimit
        let limit = calculated_limit.max(max_limit);
        if balance.checked_sub(amount).unwrap() < limit {
            return Err(ContractError::Std(StdError::GenericErr {
                msg: "Exceeds Withdrawal limits".to_string(),
            }));
        }
        Ok(RateLimit {
            percentage: self.percentage,
            period: self.period,
            last_update: current_time,
            current_limit: limit,
        })
    }
}
