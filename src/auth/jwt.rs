use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, TokenData, errors::Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

const SECRET: &[u8] = b"supersecret";

pub fn validate_token(token: &str) -> Result<Claims, Error> {
    let token_data: TokenData<Claims> = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}
