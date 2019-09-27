use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

use crate::error::Never;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Secret {
    r#type: SecretType,
    domain: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SecretType {
    Password
}

pub trait Storage: Send {
    type Error: Error;
    fn get(&self, domain: &str) -> Result<Option<Secret>, Self::Error>;
    fn set(&self, secret: Secret) -> Result<(), Self::Error>;
    fn delete(&self, domain: &str) -> Result<bool, Self::Error>;
}

impl Storage for Arc<RwLock<HashMap<String, Secret>>> {
    type Error = Never;

    fn get(&self, domain: &str) -> Result<Option<Secret>, Never> {
        let lock = self.read().unwrap();
        Ok(HashMap::get(&lock, domain).cloned())
    }

    fn set(&self, secret: Secret) -> Result<(), Never> {
        HashMap::insert(&mut self.write().unwrap(), secret.domain.clone(), secret);
        Ok(())
    }

    fn delete(&self, domain: &str) -> Result<bool, Self::Error> {
        Ok(HashMap::remove(&mut self.write().unwrap(), domain).is_some())
    }
}

