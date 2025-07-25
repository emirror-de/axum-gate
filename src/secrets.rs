//! Secrets hashing and verification models.
use crate::{
    Error,
    hashing::{HashedValue, VerificationResult},
    services::HashingService,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a secret that is bound to an [Account](crate::Account) by its [account_id](crate::Account::account_id).
///
/// The `account_id` needs to be queried from an [AccountStorage](crate::services::AccountStorageService) to be able to create a correct secret.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Secret {
    /// The [account id](crate::Account::account_id) that this secret belongs to.
    pub account_id: Uuid,
    /// The actual secret.
    pub secret: HashedValue,
}

impl Secret {
    /// Creates a new instance with the given id and secret.
    pub fn new<Hasher: HashingService>(
        account_id: &Uuid,
        plain_secret: &str,
        hasher: Hasher,
    ) -> Result<Self> {
        let secret = hasher
            .hash_value(plain_secret)
            .map_err(|e| Error::Hashing(e.to_string()))?;
        Ok(Self {
            account_id: *account_id,
            secret,
        })
    }

    /// Creates an instance given that the input secret is already a hashed value.
    pub fn from_hashed(account_id: &Uuid, hashed_secret: &HashedValue) -> Self {
        Self {
            account_id: account_id.to_owned(),
            secret: hashed_secret.to_owned(),
        }
    }

    /// Verifies the given plain value to the stored one.
    pub fn verify<Hasher: HashingService>(
        &self,
        plain_secret: &str,
        hasher: Hasher,
    ) -> Result<VerificationResult> {
        hasher.verify_value(plain_secret, &self.secret)
    }
}

#[test]
fn secret_verification() {
    use crate::hashing::Argon2Hasher;

    let id = Uuid::now_v7();
    let correct_password = "admin_password";
    let wrong_password = "admin_wrong_password";
    let secret = Secret::new(&id, correct_password, Argon2Hasher).unwrap();

    assert_eq!(
        VerificationResult::Unauthorized,
        secret.verify(wrong_password, Argon2Hasher).unwrap()
    );
    assert_eq!(
        VerificationResult::Ok,
        secret.verify(correct_password, Argon2Hasher).unwrap()
    );
}
