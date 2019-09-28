use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

use mongodb::{Bson, from_bson, to_bson};
use mongodb::coll::Collection;
use mongodb::db::ThreadedDatabase;
use r2d2::Pool;
use r2d2_mongodb::MongodbConnectionManager;

use crate::error::Never;
use mongodb::coll::options::UpdateOptions;

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

#[derive(Clone)]
pub struct MemStorage(Arc<RwLock<HashMap<String, Secret>>>);

impl MemStorage {
    #[allow(unused)]
    pub fn new() -> Self {
        MemStorage(Arc::new(RwLock::new(HashMap::new())))
    }
}

pub trait Storage: Send {
    type Error: Error;
    fn get_all(&self) -> Result<Vec<Secret>, Self::Error>;
    fn get(&self, domain: &str) -> Result<Option<Secret>, Self::Error>;
    fn set(&self, secret: Secret) -> Result<(), Self::Error>;
    fn delete(&self, domain: &str) -> Result<bool, Self::Error>;
}

impl Storage for MemStorage {
    type Error = Never;

    fn get_all(&self) -> Result<Vec<Secret>, Never> {
        let lock = self.0.read().unwrap();
        Ok(lock.values().cloned().collect())
    }

    fn get(&self, domain: &str) -> Result<Option<Secret>, Never> {
        let lock = self.0.read().unwrap();
        Ok(HashMap::get(&lock, domain).cloned())
    }

    fn set(&self, secret: Secret) -> Result<(), Never> {
        HashMap::insert(&mut self.0.write().unwrap(), secret.domain.clone(), secret);
        Ok(())
    }

    fn delete(&self, domain: &str) -> Result<bool, Self::Error> {
        Ok(HashMap::remove(&mut self.0.write().unwrap(), domain).is_some())
    }
}

#[derive(Clone)]
pub struct MongoStorage(Pool<MongodbConnectionManager>);

impl MongoStorage {
    const COLL_NAME: &'static str = "secrets";
    pub fn new(pool: Pool<MongodbConnectionManager>) -> Self {
        MongoStorage(pool)
    }
}

impl Storage for MongoStorage {
    type Error = mongodb::error::Error;

    fn get_all(&self) -> Result<Vec<Secret>, Self::Error> {
        let conn = self.0.get().map_err(|e| e.to_string())?;
        let coll: Collection = conn.collection(Self::COLL_NAME);
        coll.find(None, None)
            .and_then(|cursor|
                cursor.map(|doc| doc.and_then(|d| from_bson(Bson::from(d)).map_err(From::from)))
                    .collect::<Result<Vec<_>, _>>())
    }

    fn get(&self, domain: &str) -> Result<Option<Secret>, Self::Error> {
        let conn = self.0.get().map_err(|e| e.to_string())?;
        let coll: Collection = conn.collection(Self::COLL_NAME);
        match coll.find_one(Some(doc! { "domain": domain }), None) {
            Ok(Some(doc)) => from_bson(Bson::from(doc)).map_err(From::from).map(Some),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn set(&self, secret: Secret) -> Result<(), Self::Error> {
        let conn = self.0.get().map_err(|e| e.to_string())?;
        let coll: Collection = conn.collection(Self::COLL_NAME);
        let doc = to_bson(&secret)?;
        let mut opts = UpdateOptions::new();
        opts.upsert = Some(true);
        coll.update_one(doc! { "domain": &secret.domain }, doc! { "$set": doc }, Some(opts)).map(|_| ())
    }

    fn delete(&self, domain: &str) -> Result<bool, Self::Error> {
        let conn = self.0.get().map_err(|e| e.to_string())?;
        let coll: Collection = conn.collection(Self::COLL_NAME);
        coll.delete_one(doc! { "domain": domain }, None).map(|res| res.deleted_count > 0)
    }
}
