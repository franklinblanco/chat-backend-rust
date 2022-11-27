use serde::{Serialize, Deserialize};


/// A user for chats, mainly used for authentication
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub keyset: KeySet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeySet {
    pub private: String,
    pub public: String
}