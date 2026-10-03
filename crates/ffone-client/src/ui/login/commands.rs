use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}
