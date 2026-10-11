#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct TrustTrail;

#[contractimpl]
impl TrustTrail {
    pub fn ping(_env: Env) -> u32 {
        1
    }
}

#[cfg(test)]
mod test {
    use super::{TrustTrail, TrustTrailClient};
    use soroban_sdk::Env;

    #[test]
    fn ping_returns_one() {
        let env = Env::default();
        let contract_id = env.register(TrustTrail, ());
        let client = TrustTrailClient::new(&env, &contract_id);

        assert_eq!(client.ping(), 1);
    }
}
