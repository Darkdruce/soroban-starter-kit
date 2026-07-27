// `#[contracttype]` generates undocumented public associated items.
#![allow(missing_docs)]

use soroban_sdk::{Address, contracttype};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    WrappedToken,
    UnderlyingToken,
    TotalWrapped,
    /// Instance storage – whether the contract is paused (`bool`). Only used
    /// when the `pausable` feature is enabled.
    Paused,
    /// Instance storage – optional cap on the cumulative amount any single
    /// address may ever wrap (`i128`). Absent means uncapped.
    MaxWrapPerAddress,
    /// Persistent storage – cumulative amount wrapped so far by a given
    /// `Address` (`i128`). Only tracked when `MaxWrapPerAddress` is set.
    WrappedByAddress(Address),
}

#[cfg(test)]
mod discriminant_tests {
    use super::*;
    use soroban_sdk::{Env, testutils::Address as _};

    // In Soroban, #[contracttype] enums use the variant NAME as the XDR storage discriminant.
    // NEVER rename, reorder, or remove variants — doing so will corrupt on-chain storage for
    // any live deployment. To add a new key, append it at the END of the enum definition.
    //
    // This exhaustive match is the primary guard: it causes a COMPILE ERROR if a variant is
    // renamed or removed, and a non-exhaustive warning if one is added without updating here.
    fn wrapped_token_data_key_index(key: &DataKey) -> u32 {
        match key {
            DataKey::Admin => 0,
            DataKey::WrappedToken => 1,
            DataKey::UnderlyingToken => 2,
            DataKey::TotalWrapped => 3,
            DataKey::Paused => 4,
            DataKey::MaxWrapPerAddress => 5,
            DataKey::WrappedByAddress(_) => 6,
        }
    }

    #[test]
    fn data_key_discriminants_are_stable() {
        let env = Env::default();
        let addr = Address::generate(&env);

        assert_eq!(wrapped_token_data_key_index(&DataKey::Admin), 0);
        assert_eq!(wrapped_token_data_key_index(&DataKey::WrappedToken), 1);
        assert_eq!(wrapped_token_data_key_index(&DataKey::UnderlyingToken), 2);
        assert_eq!(wrapped_token_data_key_index(&DataKey::TotalWrapped), 3);
        assert_eq!(wrapped_token_data_key_index(&DataKey::Paused), 4);
        assert_eq!(wrapped_token_data_key_index(&DataKey::MaxWrapPerAddress), 5);
        assert_eq!(
            wrapped_token_data_key_index(&DataKey::WrappedByAddress(addr)),
            6
        );
    }
}
