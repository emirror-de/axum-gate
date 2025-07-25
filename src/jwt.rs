//! Claims and JWT models.
use crate::Error;
use crate::jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use crate::services::CodecService;

use std::collections::HashSet;
use std::marker::PhantomData;

use anyhow::{Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::skip_serializing_none;

/// Registered/reserved claims by IANA/JWT spec, see
/// [auth0](https://auth0.com/docs/secure/tokens/json-web-tokens/json-web-token-claims) for more
/// information.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[skip_serializing_none]
pub struct RegisteredClaims {
    /// Issuer of the JWT
    #[serde(rename = "iss")]
    pub issuer: String,
    /// Subject of the JWT (the user)
    #[serde(rename = "sub")]
    pub subject: Option<String>,
    /// Recipient for which the JWT is intended
    #[serde(rename = "aud")]
    pub audience: Option<HashSet<String>>,
    /// Time after which the JWT expires
    #[serde(rename = "exp")]
    pub expiration_time: u64,
    /// Time before which the JWT must not be accepted for processing
    #[serde(rename = "nbf")]
    pub not_before_time: Option<u64>,
    /// Time at which the JWT was issued; can be used to determine age of the JWT
    #[serde(rename = "iat")]
    pub issued_at_time: u64,
    /// Unique identifier; can be used to prevent the JWT from being replayed (allows a token to be used only once)
    #[serde(rename = "jti")]
    pub jwt_id: Option<String>,
}

impl RegisteredClaims {
    /// Initializes the claims. Automatically sets the `issued_at_time` to `Utc::now`.
    pub fn new(issuer: &str, expiration_time: u64) -> Self {
        Self {
            issuer: issuer.to_string(),
            subject: None,
            audience: None,
            expiration_time,
            not_before_time: None,
            issued_at_time: Utc::now().timestamp() as u64,
            jwt_id: None,
        }
    }
}

/// Combination of claims used within `axum-gate` and encoded with [JsonWebToken] codec.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JwtClaims<CustomClaims> {
    /// The registered claims of a JWT.
    #[serde(flatten)]
    pub registered_claims: RegisteredClaims,
    /// Your custom claims that are added to the JWT.
    #[serde(flatten)]
    pub custom_claims: CustomClaims,
}

impl<CustomClaims> JwtClaims<CustomClaims> {
    /// Creates new claims with the given registered claims.
    pub fn new(custom_claims: CustomClaims, registered_claims: RegisteredClaims) -> Self {
        Self {
            custom_claims,
            registered_claims,
        }
    }

    /// Checks whether the issuer equals to the given value.
    pub fn has_issuer(&self, issuer: &str) -> bool {
        self.registered_claims.issuer == issuer
    }
}

/// Options to configure the [JsonWebToken] codec.
pub struct JsonWebTokenOptions {
    /// Key for encoding.
    pub enc_key: EncodingKey,
    /// Key for decoding.
    pub dec_key: DecodingKey,
    /// The header used for encoding.
    pub header: Option<Header>,
    /// Validation options.
    pub validation: Option<Validation>,
}

impl Default for JsonWebTokenOptions {
    /// Creates a random, alphanumeric 60 char key and uses it for en- and decoding (symmetric).
    /// [Header] and [Validation] are set with its default values.
    fn default() -> Self {
        use rand::{Rng, distr::Alphanumeric, rng};

        let authentication_secret: String = rng()
            .sample_iter(&Alphanumeric)
            .take(60)
            .map(char::from)
            .collect();
        Self {
            enc_key: EncodingKey::from_secret(authentication_secret.as_bytes()),
            dec_key: DecodingKey::from_secret(authentication_secret.as_bytes()),
            header: Some(Header::default()),
            validation: Some(Validation::default()),
        }
    }
}

impl JsonWebTokenOptions {
    /// Adds the given encoding key.
    pub fn with_encoding_key(self, enc_key: EncodingKey) -> Self {
        Self { enc_key, ..self }
    }

    /// Adds the given decoding key.
    pub fn with_decoding_key(self, dec_key: DecodingKey) -> Self {
        Self { dec_key, ..self }
    }

    /// Adds the given header.
    pub fn with_header(self, header: Header) -> Self {
        Self {
            header: Some(header),
            ..self
        }
    }

    /// Adds the given validation.
    pub fn with_validation(self, validation: Validation) -> Self {
        Self {
            validation: Some(validation),
            ..self
        }
    }
}

/// Encrypts using the given keys as JWT using [jsonwebtoken].
#[derive(Clone)]
pub struct JsonWebToken<P> {
    /// Key for encoding.
    enc_key: EncodingKey,
    /// Key for decoding.
    dec_key: DecodingKey,
    /// The header used for encoding.
    pub header: Header,
    /// Validation options for the JWT.
    pub validation: Validation,
    phantom_payload: PhantomData<P>,
}

impl<P> JsonWebToken<P> {
    /// Creates a new instance with the given options.
    pub fn new_with_options(options: JsonWebTokenOptions) -> Self {
        let JsonWebTokenOptions {
            enc_key,
            dec_key,
            header,
            validation,
        } = options;
        Self {
            enc_key,
            dec_key,
            header: header.unwrap_or(Header::default()),
            validation: validation.unwrap_or(Validation::default()),
            phantom_payload: PhantomData,
        }
    }
}

impl<P> Default for JsonWebToken<P> {
    fn default() -> Self {
        Self::new_with_options(JsonWebTokenOptions::default())
    }
}

impl<P> CodecService for JsonWebToken<P>
where
    P: Serialize + DeserializeOwned + Clone,
{
    type Payload = P;
    fn encode(&self, payload: &Self::Payload) -> Result<Vec<u8>> {
        let web_token = jsonwebtoken::encode(&self.header, payload, &self.enc_key)
            .map_err(|e| Error::Codec(format!("{e}")))?;
        Ok(web_token.as_bytes().to_vec())
    }
    /// Decodes the given value.
    ///
    /// # Errors
    /// Returns an error if the header stored in [JsonWebToken] does not match the decoded value.
    /// The header can be retrieved from [JsonWebToken::header].
    fn decode(&self, encoded_value: &[u8]) -> Result<Self::Payload> {
        let claims = jsonwebtoken::decode::<Self::Payload>(
            &String::from_utf8_lossy(encoded_value),
            &self.dec_key,
            &self.validation,
        )
        .map_err(|e| Error::Codec(format!("{e}")))?;

        if self.header != claims.header {
            return Err(anyhow!(Error::Codec(
                "Header of the decoded value does not match the one used for encoding.".to_string()
            )));
        }

        Ok(claims.claims)
    }
}
