use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Deserialize, Serialize, FromRow)]
pub struct Directory {
    pub name: String,
    pub path: String,
}

