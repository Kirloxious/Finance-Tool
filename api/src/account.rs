use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::Routable;

trait FromRow: Sized {
    fn from_row(row: &rusqlite::Row) -> Result<Box<dyn BankAccount>, rusqlite::Error>;
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum AccountType {
    Savings,
    Credit,
    Chequing,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct InvalidAccountType;
impl fmt::Display for InvalidAccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid account type")
    }
}

impl FromStr for AccountType {
    type Err = InvalidAccountType;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "savings" => Ok(AccountType::Savings),
            "credit" => Ok(AccountType::Credit),
            "chequing" => Ok(AccountType::Chequing),
            "checking" => Ok(AccountType::Chequing),
            _ => Err(InvalidAccountType),
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountType::Savings => write!(f, "Savings"),
            AccountType::Credit => write!(f, "Credit"),
            AccountType::Chequing => write!(f, "Chequing"),
            AccountType::Unknown => write!(f, "Unknown"),
        }
    }
}

pub fn bank_account_from_row(row: &rusqlite::Row) -> Result<Box<dyn BankAccount>, rusqlite::Error> {
    let account_type =
        AccountType::from_str(&row.get::<_, String>(1)?).unwrap_or(AccountType::Unknown);
    match account_type {
        AccountType::Savings => SavingsAccount::from_row(row),
        AccountType::Credit => CreditAccount::from_row(row),
        AccountType::Chequing => ChequingAccount::from_row(row),
        AccountType::Unknown => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Account {
    Savings(SavingsAccount),
    Credit(CreditAccount),
    Chequing(ChequingAccount),
}

pub trait BankAccount: Send + Sync {
    fn user_id(&self) -> i64;
    fn deposit(&mut self, amount: f64);
    fn withdraw(&mut self, amount: f64);
    fn balance(&self) -> f64;
    fn account_type(&self) -> AccountType;
    fn account_number(&self) -> &i64;
    fn credit_limit(&self) -> f64;
    fn interest_rate(&self) -> f64;
    fn set_balance(&mut self, balance: f64);
    fn set_credit_limit(&mut self, limit: f64);
    fn as_enum(self: Box<Self>) -> Account;
}

#[derive(Serialize, Deserialize)]
pub struct SavingsAccount {
    pub user_id: i64,
    pub account_number: i64,
    pub balance: f64,
    pub interest_rate: f64, // will be hard coded
}

impl SavingsAccount {
    pub fn new(
        user_id: i64,
        account_number: i64,
        balance: f64,
        interest_rate: f64,
    ) -> SavingsAccount {
        SavingsAccount {
            user_id,
            account_number,
            balance,
            interest_rate,
        }
    }
}

impl BankAccount for SavingsAccount {
    fn account_number(&self) -> &i64 {
        &self.account_number
    }
    fn account_type(&self) -> AccountType {
        AccountType::Savings
    }
    fn balance(&self) -> f64 {
        self.balance
    }

    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }

    fn withdraw(&mut self, amount: f64) {
        self.balance -= amount;
    }

    fn interest_rate(&self) -> f64 {
        self.interest_rate
    }

    fn user_id(&self) -> i64 {
        self.user_id
    }

    fn credit_limit(&self) -> f64 {
        0.0
    }
    fn set_balance(&mut self, balance: f64) {
        self.balance = balance;
    }

    fn set_credit_limit(&mut self, _limit: f64) {
        todo!();
    }

    fn as_enum(self: Box<Self>) -> Account {
        Account::Savings(*self)
    }
}

impl FromRow for SavingsAccount {
    fn from_row(row: &rusqlite::Row) -> Result<Box<dyn BankAccount>, rusqlite::Error> {
        Ok(Box::new(SavingsAccount::new(
            row.get(0)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        )))
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreditAccount {
    pub user_id: i64,
    pub account_number: i64,
    pub balance_owed: f64,
    pub credit_limit: f64, // will be hard coded
}

impl CreditAccount {
    pub fn new(
        user_id: i64,
        account_number: i64,
        balance_owed: f64,
        credit_limit: f64,
    ) -> CreditAccount {
        CreditAccount {
            user_id,
            account_number,
            balance_owed,
            credit_limit,
        }
    }

    pub fn credit_limit(&self) -> f64 {
        self.credit_limit
    }
}

impl BankAccount for CreditAccount {
    fn account_number(&self) -> &i64 {
        &self.account_number
    }

    fn account_type(&self) -> AccountType {
        AccountType::Credit
    }

    fn balance(&self) -> f64 {
        self.balance_owed
    }

    fn deposit(&mut self, amount: f64) {
        self.balance_owed += amount;
    }

    fn withdraw(&mut self, amount: f64) {
        self.balance_owed -= amount;
    }

    fn user_id(&self) -> i64 {
        self.user_id
    }

    fn interest_rate(&self) -> f64 {
        0.0
    }

    fn credit_limit(&self) -> f64 {
        self.credit_limit
    }

    fn set_balance(&mut self, balance: f64) {
        self.balance_owed = balance;
    }

    fn set_credit_limit(&mut self, limit: f64) {
        self.credit_limit = limit;
    }
    fn as_enum(self: Box<Self>) -> Account {
        Account::Credit(*self)
    }
}

impl FromRow for CreditAccount {
    fn from_row(row: &rusqlite::Row) -> Result<Box<dyn BankAccount>, rusqlite::Error> {
        Ok(Box::new(CreditAccount::new(
            row.get(0)?,
            row.get(2)?,
            row.get(3)?,
            row.get(5)?,
        )))
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChequingAccount {
    pub user_id: i64,
    pub account_number: i64,
    pub balance: f64,
}

impl ChequingAccount {
    pub fn new(user_id: i64, account_number: i64, balance: f64) -> ChequingAccount {
        ChequingAccount {
            user_id,
            account_number,
            balance,
        }
    }
}

impl BankAccount for ChequingAccount {
    fn account_number(&self) -> &i64 {
        &self.account_number
    }
    fn account_type(&self) -> AccountType {
        AccountType::Chequing
    }
    fn balance(&self) -> f64 {
        self.balance
    }
    fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }
    fn withdraw(&mut self, amount: f64) {
        self.balance -= amount;
    }

    fn user_id(&self) -> i64 {
        self.user_id
    }

    fn interest_rate(&self) -> f64 {
        0.0
    }

    fn credit_limit(&self) -> f64 {
        0.0
    }

    fn set_balance(&mut self, balance: f64) {
        self.balance = balance;
    }

    fn set_credit_limit(&mut self, _limit: f64) {
        todo!();
    }

    fn as_enum(self: Box<Self>) -> Account {
        Account::Chequing(*self)
    }
}

impl FromRow for ChequingAccount {
    fn from_row(row: &rusqlite::Row) -> Result<Box<dyn BankAccount>, rusqlite::Error> {
        Ok(Box::new(ChequingAccount::new(
            row.get(0)?,
            row.get(2)?,
            row.get(3)?,
        )))
    }
}
