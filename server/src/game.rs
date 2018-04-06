use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait Game {
    type Player: Clone + Default + Send + Serialize + DeserializeOwned;
}