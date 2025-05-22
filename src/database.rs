use rusqlite::named_params;
use rusqlite::Connection;
use rusqlite::Result;

use crate::account::Account;
use crate::account::AccountType;
use crate::account::BankAccount;
use crate::transaction::Transaction;
use crate::user::User;

pub struct Database {
    _db_path: &'static str,
    connection: Connection,
}

impl Database {
    pub fn new(path: &'static str) -> Result<Database> {
        let conn = Connection::open(path)?;
        Ok(Database {
            _db_path: path,
            connection: conn,
        })
    }

    fn get_connection(&self) -> &Connection {
        &self.connection
    }

    pub fn close_connection(self) -> Result<()> {
        self.connection.close().expect("Failed to close connection");
        Ok(())
    }

    pub fn reset_values(&self) -> Result<()> {
        let conn = self.get_connection();
        conn.execute("DELETE FROM Users", ())?;
        conn.execute("DELETE FROM Account", ())?;
        conn.execute("DELETE FROM Transactions", ())?;
        Ok(())
    }

    pub fn insert_transaction(&self, transaction: &Transaction) -> Result<()> {
        let conn = self.get_connection();
        let mut statement = conn.prepare("INSERT INTO Transactions (user_id, account_type, account_number, transaction_date, cheque_number, description_1, description_2, cad, usd, category) VALUES (?,?,?,?,?,?,?,?,?,?)")?;
        match statement.execute((
            &transaction.user_id,
            &transaction.account_type.to_string(),
            &transaction.account_number,
            &transaction.transaction_date,
            &transaction.cheque_number,
            &transaction.description_1,
            &transaction.description_2,
            &transaction.cad,
            &transaction.usd,
            &transaction.category,
        )) {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Failed to insert transaction: {}", e);
                println!("Error:");
                Err(e)
            }
        }
    }

    pub fn batch_insert_transactions(&self, transactions: &Vec<Transaction>) -> Result<()> {
        let conn = self.get_connection();
        let mut statement = conn.prepare("INSERT INTO Transactions (user_id, account_type, account_number, transaction_date, cheque_number, description_1, description_2, cad, usd, category) VALUES (?,?,?,?,?,?,?,?,?,?)")?;

        for transaction in transactions {
            statement.execute((
                &transaction.user_id,
                &transaction.account_type.to_string(),
                &transaction.account_number,
                &transaction.transaction_date,
                &transaction.cheque_number,
                &transaction.description_1,
                &transaction.description_2,
                &transaction.cad,
                &transaction.usd,
                &transaction.category,
            ))?;
        }

        Ok(())
    }

    pub fn insert_user(&self, user: &User) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "INSERT INTO Users (user_id, name) VALUES (?,?)",
            (&user.id, &user.name),
        )?;
        Ok(())
    }

    pub fn get_user_by_name(&self, name: &str) -> Result<User> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare("SELECT * FROM Users WHERE name = :name")?;
        let mut rows = stmt.query_map(named_params! {":name": name}, |row| {
            Ok(User::from_row(row)?)
        })?;
        Ok(rows.next().unwrap()?)
    }

    pub fn insert_account<A: BankAccount>(&self, account: &A) -> Result<()> {
        let conn = self.get_connection();

        conn.execute("INSERT INTO Account (user_id, account_type, account_number, balance, interest_rate, credit_limit) VALUES (?,?,?,?,?,?)", 
        (&account.user_id(), &account.account_type().to_string(), &account.account_number(), &account.balance(), &account.interest_rate(), &account.credit_limit()))?;

        Ok(())
    }

    pub fn update_account(&self, account: &Account) -> Result<()> {
        let conn = self.get_connection();
        conn.execute(
            "UPDATE Account SET balance = ?, interest_rate = ?, credit_limit = ? WHERE account_number = ?",
            (&account.balance(), &account.interest_rate(), &account.credit_limit(), &account.account_number()),
        )?;
        Ok(())
    }

    pub fn account_exists(&self, account_number: &i64) -> Result<bool> {
        let conn = self.get_connection();
        let mut stmt = conn
            .prepare("SELECT account_number FROM Account WHERE account_number = :account_number")?;
        let mut rows = stmt
            .query_map(named_params! {":account_number": account_number}, |row| {
                Ok(row.get::<_, String>(0))
            })?;
        Ok(rows.next().is_some())
    }

    pub fn get_account(&self, account_number: &String) -> Result<Account> {
        let conn = self.get_connection();
        let mut stmt =
            conn.prepare("SELECT * FROM Account WHERE account_number = :account_number")?;
        let mut rows = stmt
            .query_map(named_params! {":account_number": (account_number)}, |row| {
                Ok(Account::from_row(row)?)
            })?;
        Ok(rows.next().unwrap()?)
    }

    pub fn get_accounts_by_user(&self, user_id: i64) -> Result<Vec<Account>> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare("SELECT * FROM Account WHERE user_id = :user_id")?;
        let rows = stmt.query_map(named_params! {":user_id": user_id}, |row| {
            Ok(Account::from_row(row)?)
        })?;

        let mut accounts = Vec::new();
        for row in rows {
            accounts.push(row?);
        }
        Ok(accounts)
    }

    pub fn get_account_number_by_type(
        &self,
        user_id: i64,
        account_type: &AccountType,
    ) -> Result<i64> {
        let conn = self.get_connection();
        let mut stmt = conn.prepare("SELECT account_number FROM Account WHERE user_id = :user_id AND account_type = :account_type")?;
        let mut rows = stmt.query_map(
            named_params! {":user_id": user_id, ":account_type": account_type.to_string()},
            |row| Ok(row.get::<_, i64>(0)),
        )?;
        if let Some(row_result) = rows.next() {
            match row_result {
                Ok(account_number) => return Ok(account_number?),
                Err(e) => return Err(e),
            }
        } else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
    }

    // Used once to create the database schema
    pub fn _execute_schema(&self) -> Result<()> {
        let conn = self.get_connection();

        conn.execute("PRAGMA foreign_keys = ON;", ())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Users (
            user_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Account (
                user_id INTEGER NOT NULL,
                account_type TEXT,
                account_number INTEGER PRIMARY KEY,
                balance REAL,
                interest_rate REAL,
                credit_limit REAL,

                FOREIGN KEY(user_id) REFERENCES Users(user_id) ON DELETE CASCADE ON UPDATE CASCADE
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS Transactions(
            transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            account_number INTEGER NOT NULL,
            account_type TEXT NOT NULL,
            transaction_date TEXT NOT NULL,
            cheque_number TEXT,
            description_1 TEXT,
            description_2 TEXT,
            cad REAL,
            usd REAL,

            FOREIGN KEY(user_id) REFERENCES Users(user_id) ON DELETE CASCADE ON UPDATE CASCADE,
            FOREIGN KEY(account_number) REFERENCES Account(account_number) ON DELETE CASCADE ON UPDATE CASCADE
        
        )",
            (),
        )?;

        Ok(())
    }
}
