use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub id: i64,
}

impl User {
    pub fn new(name: String) -> User {
        User {
            name,
            id: User::random_id(),
        }
    }

    // random 6 digit id
    fn random_id() -> i64 {
        let mut rng = rand::rng();
        rng.random_range(100_000..999_999) as i64
    }

    pub fn from_row(row: &rusqlite::Row) -> Result<User, rusqlite::Error> {
        Ok(User {
            name: row.get(1)?,
            id: row.get(0)?,
        })
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            name: "".into(),
            id: 0,
        }
    }
}

impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
