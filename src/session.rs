use crate::database::Database;
use crate::auth::{User, Permissions};

use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub user: String,
    pub permissions: Permissions,
}

