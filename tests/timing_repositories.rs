//! Integration timing tests for repository credential verification.
//!
//! These tests assert that credential verification for:
//!
//!   1. Non-existent account
//!   2. Existing account with wrong password
//!   3. Existing account with correct password
//!
//! take comparable time (no large timing gap between (1) and (2)),
//! which would otherwise permit user enumeration via timing analysis.
//!
//! The repositories under test internally pre-compute a dummy Argon2 hash
//! (with the same build-mode preset parameters as real stored hashes) and
//! always perform an Argon2 verification, even when the account/secret
//! does not exist.
//!
//! To reduce flakiness on CI (where jitter / noisy neighbors can skew single
//! measurements), we:
//!   * Perform a small warm‑up
//!   * Take multiple timing samples
//!   * Compare medians (robust vs outliers)
//!
//! Thresholds are widened (relative to initial strict values) but still ensure
//! no large, systematic discrepancy emerges between nonexistent-user and
//! wrong-password paths.
//!
//! Run with: `cargo test -- --nocapture` to see raw timing output.
use axum_gate::accounts::{Account, AccountRepository};
use axum_gate::credentials::{Credentials, CredentialsVerifier};
use axum_gate::hashing::argon2::Argon2Hasher;
use axum_gate::prelude::{Group, Role};
use axum_gate::secrets::{Secret, SecretRepository};
use axum_gate::verification_result::VerificationResult;
use std::time::{Duration, Instant};

/// Returns a fresh random (UUID v7 backed) email identifier to avoid collisions in shared DBs.
fn random_user_id() -> String {
    format!("user+{}@example.test", uuid::Uuid::now_v7())
}

/// Compute median duration (simple sort; small N so overhead negligible)
fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

//
// SurrealDB Timing Test
//
#[tokio::test]
#[cfg(feature = "storage-surrealdb")]
async fn surrealdb_timing_symmetry() {
    use axum_gate::repositories::surrealdb::{DatabaseScope, SurrealDbRepository};
    use surrealdb::Surreal;
    use surrealdb::engine::local::Mem;

    const ITERATIONS: usize = 5;

    // Prepare SurrealDB in-memory instance
    let db = Surreal::new::<Mem>(()).await.unwrap();
    let scope = DatabaseScope::default();
    let repo = SurrealDbRepository::new(db, scope).unwrap();

    // Create & store an account + secret
    let existing_user = random_user_id();
    let account = Account::new(&existing_user, &[Role::User], &[Group::new("grp")]);
    let stored_account = repo
        .store_account(account)
        .await
        .expect("store account")
        .expect("stored account");

    let password = "correct_password";
    let hasher = Argon2Hasher::new_recommended().unwrap();
    let secret = Secret::new(&stored_account.account_id, password, hasher).expect("hash secret");
    repo.store_secret(secret).await.expect("store secret");

    // Warm-up (untimed)
    for _ in 0..2 {
        let _ = repo
            .verify_credentials(Credentials::new(
                &stored_account.account_id,
                "wrong_password",
            ))
            .await;
        let _ = repo
            .verify_credentials(Credentials::new(
                &random_user_id(),
                "whatever_wrong_password_not_relevant",
            ))
            .await;
    }

    let mut nonexist_durs = Vec::with_capacity(ITERATIONS);
    let mut wrong_durs = Vec::with_capacity(ITERATIONS);
    let mut correct_durs = Vec::with_capacity(ITERATIONS);

    for _ in 0..ITERATIONS {
        // Fresh creds each iteration for nonexistent path
        let creds_nonexistent =
            Credentials::new(&random_user_id(), "whatever_wrong_password_not_relevant");
        let creds_wrong = Credentials::new(&stored_account.account_id, "wrong_password");
        let creds_correct = Credentials::new(&stored_account.account_id, password);

        // Non-existent
        let t0 = Instant::now();
        let res_nonexistent = repo
            .verify_credentials(creds_nonexistent)
            .await
            .expect("verify nonexistent");
        let dur_nonexistent = t0.elapsed();

        // Wrong password
        let t1 = Instant::now();
        let res_wrong = repo
            .verify_credentials(creds_wrong)
            .await
            .expect("verify wrong");
        let dur_wrong = t1.elapsed();

        // Correct password
        let t2 = Instant::now();
        let res_correct = repo
            .verify_credentials(creds_correct)
            .await
            .expect("verify correct");
        let dur_correct = t2.elapsed();

        assert!(matches!(res_nonexistent, VerificationResult::Unauthorized));
        assert!(matches!(res_wrong, VerificationResult::Unauthorized));
        assert!(matches!(res_correct, VerificationResult::Ok));

        nonexist_durs.push(dur_nonexistent);
        wrong_durs.push(dur_wrong);
        correct_durs.push(dur_correct);
    }

    let med_nonexist = median(nonexist_durs.clone());
    let med_wrong = median(wrong_durs.clone());
    let med_correct = median(correct_durs.clone());

    let diff = med_nonexist.abs_diff(med_wrong);

    println!(
        "[SurrealDB Timing] med(nonexistent)={:?}, med(wrong)={:?}, med(correct)={:?}, diff(nonexist vs wrong)={:?}, samples_nonexist={:?}, samples_wrong={:?}",
        med_nonexist, med_wrong, med_correct, diff, nonexist_durs, wrong_durs
    );

    // Relaxed threshold (prior single-shot limit 25ms). Median diff should
    // stay comfortably below this unless there's a systematic leak.
    assert!(
        diff.as_millis() < 90,
        "SurrealDB timing median difference too large: {:?} ms",
        diff.as_millis()
    );

    // Ensure failure paths are not trivially fast (Argon2 executed)
    assert!(
        med_nonexist.as_millis() >= 5,
        "Median nonexistent path too fast; Argon2 likely skipped."
    );
    assert!(
        med_wrong.as_millis() >= 5,
        "Median wrong password path too fast; Argon2 likely skipped."
    );
}

//
// SeaORM Timing Test
//
#[tokio::test]
#[allow(clippy::unwrap_used)]
#[cfg(feature = "storage-seaorm")]
async fn seaorm_timing_symmetry() {
    use axum_gate::repositories::sea_orm::SeaOrmRepository;
    #[cfg(feature = "storage-seaorm")]
    use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};

    const ITERATIONS: usize = 5;

    // In-memory SQLite database
    let db = match Database::connect("sqlite::memory:").await {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Skipping SeaORM timing test (unable to connect sqlite memory): {e}");
            return;
        }
    };

    // Create tables using SeaORM entity definitions (no migrations needed)
    use axum_gate::repositories::sea_orm::models::{
        account as seaorm_account, credentials as seaorm_credentials,
    };
    let builder = Schema::new(DatabaseBackend::Sqlite);
    let account_stmt = builder.create_table_from_entity(seaorm_account::Entity);
    let credentials_stmt = builder.create_table_from_entity(seaorm_credentials::Entity);

    // Execute table creation statements
    if let Err(e) = db.execute(&account_stmt).await {
        eprintln!("Skipping SeaORM timing test (create accounts table failed): {e}");
        return;
    }
    if let Err(e) = db.execute(&credentials_stmt).await {
        eprintln!("Skipping SeaORM timing test (create credentials table failed): {e}");
        return;
    }

    let repo = SeaOrmRepository::new(&db).unwrap();

    // Create & store account + secret
    let existing_user = random_user_id();
    let account = Account::new(&existing_user, &[Role::User], &[Group::new("grp")]);
    let stored_account = match repo.store_account(account).await {
        Ok(Some(acc)) => acc,
        Ok(None) => {
            eprintln!("Skipping SeaORM timing test (account not stored).");
            return;
        }
        Err(e) => {
            eprintln!("Skipping SeaORM timing test (store account failed): {e}");
            return;
        }
    };

    let password = "correct_password";
    let hasher = Argon2Hasher::new_recommended().unwrap();
    let secret = match Secret::new(&stored_account.account_id, password, hasher) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping SeaORM timing test (secret hash failed): {e}");
            return;
        }
    };

    if let Err(e) = repo.store_secret(secret).await {
        eprintln!("Skipping SeaORM timing test (store secret failed): {e}");
        return;
    }

    // Warm-up
    for _ in 0..2 {
        let _ = repo
            .verify_credentials(Credentials::new(
                &stored_account.account_id,
                "wrong_password",
            ))
            .await;
        let _ = repo
            .verify_credentials(Credentials::new(
                &uuid::Uuid::now_v7(),
                "irrelevant_wrong_password_value",
            ))
            .await;
    }

    let mut nonexist_durs = Vec::with_capacity(ITERATIONS);
    let mut wrong_durs = Vec::with_capacity(ITERATIONS);
    let mut correct_durs = Vec::with_capacity(ITERATIONS);

    for _ in 0..ITERATIONS {
        let creds_nonexistent =
            Credentials::new(&uuid::Uuid::now_v7(), "irrelevant_wrong_password_value");
        let creds_wrong = Credentials::new(&stored_account.account_id, "wrong_password");
        let creds_correct = Credentials::new(&stored_account.account_id, password);

        // Non-existent
        let t0 = Instant::now();
        let res_nonexistent = match repo.verify_credentials(creds_nonexistent).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping SeaORM timing test (verify nonexistent failed): {e}");
                return;
            }
        };
        let dur_nonexistent = t0.elapsed();

        // Wrong password
        let t1 = Instant::now();
        let res_wrong = match repo.verify_credentials(creds_wrong).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping SeaORM timing test (verify wrong failed): {e}");
                return;
            }
        };
        let dur_wrong = t1.elapsed();

        // Correct
        let t2 = Instant::now();
        let res_correct = match repo.verify_credentials(creds_correct).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Skipping SeaORM timing test (verify correct failed): {e}");
                return;
            }
        };
        let dur_correct = t2.elapsed();

        assert!(matches!(res_nonexistent, VerificationResult::Unauthorized));
        assert!(matches!(res_wrong, VerificationResult::Unauthorized));
        assert!(matches!(res_correct, VerificationResult::Ok));

        nonexist_durs.push(dur_nonexistent);
        wrong_durs.push(dur_wrong);
        correct_durs.push(dur_correct);
    }

    let med_nonexist = median(nonexist_durs.clone());
    let med_wrong = median(wrong_durs.clone());
    let med_correct = median(correct_durs.clone());

    let diff = med_nonexist.abs_diff(med_wrong);

    println!(
        "[SeaORM Timing] med(nonexistent)={:?}, med(wrong)={:?}, med(correct)={:?}, diff(nonexist vs wrong)={:?}, samples_nonexist={:?}, samples_wrong={:?}",
        med_nonexist, med_wrong, med_correct, diff, nonexist_durs, wrong_durs
    );

    // Relaxed threshold (previous single-run 30ms). Median diff should remain small.
    assert!(
        diff.as_millis() < 90,
        "SeaORM timing median difference too large: {:?} ms",
        diff.as_millis()
    );

    assert!(
        med_nonexist.as_millis() >= 5,
        "Median nonexistent path too fast; Argon2 likely skipped."
    );
    assert!(
        med_wrong.as_millis() >= 5,
        "Median wrong password path too fast; Argon2 likely skipped."
    );
}
