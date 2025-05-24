use std::str::FromStr;

use crate::{
    account::{AccountType, BankAccount, ChequingAccount, CreditAccount, SavingsAccount},
    calculate_hash,
};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub user_id: i64,
    pub account_type: AccountType,
    pub account_number: i64,
    pub transaction_date: String,
    pub cheque_number: String,
    pub description_1: String,
    pub description_2: String,
    pub cad: f64,
    pub usd: f64,
    pub category: String,
}

impl Transaction {
    pub fn dummy() -> Transaction {
        Transaction {
            user_id: 0,
            account_type: AccountType::Credit,
            account_number: i64::MAX,
            transaction_date: "".to_string(),
            cheque_number: "".to_string(),
            description_1: "TIM HORTONS #7525, NEPEAN".to_string(),
            description_2: "".to_string(),
            cad: 0.0,
            usd: 0.0,
            category: "".to_string(),
        }
    }

    pub fn from_rbc_csv(user_id: i64, line: String) -> Transaction {
        let parts = line.split(',').collect::<Vec<_>>();
        let account_type = parts[0].to_string().to_lowercase();
        let real_account_type = AccountType::from_str(match account_type.as_str() {
            "visa" => "credit",
            _ => account_type.as_str(),
        })
        .unwrap_or_else(|e| {
            println!("Failed to parse account type: {}", e);
            AccountType::Unknown
        });

        Transaction {
            user_id,
            account_type: real_account_type,
            account_number: calculate_hash(&parts[1].to_string().replace("-", "")),
            transaction_date: parts[2].to_string(),
            cheque_number: parts[3].to_string(),
            description_1: parts[4].trim().replace("\"", "").to_string(),
            description_2: parts[5].trim().replace("\"", "").to_string(),
            cad: parts[6].parse::<f64>().unwrap_or(0.0),
            usd: parts[7].parse::<f64>().unwrap_or(0.0),
            category: "".to_string(),
        }
    }

    pub fn from_cibc_csv(user_id: i64, account_type: AccountType, line: String) -> Transaction {
        let parts = line.split(',').collect::<Vec<_>>();
        Transaction {
            user_id,
            account_type,
            account_number: calculate_hash(&parts[0].to_string().replace("-", "")),
            transaction_date: parts[1].to_string(),
            cheque_number: parts[2].to_string(),
            description_1: parts[3].trim().replace("\"", "").to_string(),
            description_2: parts[4].trim().replace("\"", "").to_string(),
            cad: parts[5].parse::<f64>().unwrap_or(0.0),
            usd: parts[6].parse::<f64>().unwrap_or(0.0),
            category: "".to_string(),
        }
    }

    pub fn extract_account(&self) -> Option<Box<dyn BankAccount>> {
        match self.account_type {
            AccountType::Savings => Some(Box::new(SavingsAccount::new(
                self.user_id,
                self.account_number,
                0.0,
                0.0,
            ))),
            AccountType::Credit => Some(Box::new(CreditAccount::new(
                self.user_id,
                self.account_number,
                0.0,
                0.0,
            ))),
            AccountType::Chequing => Some(Box::new(ChequingAccount::new(
                self.user_id,
                self.account_number,
                0.0,
            ))),
            AccountType::Unknown => None,
        }
    }

    pub fn extract_account_info(&self) -> (&i64, &AccountType) {
        (&self.account_number, &self.account_type)
    }

    pub fn seriazlize_to_catergorize(&self) -> serde_json::Value {
        let name = String::from(
            format!(
                "{}: {} {}",
                self.account_type, self.description_1, self.description_2
            )
            .trim(),
        );

        let mut merchant =
            String::from(format!("{} {}", self.description_1, self.description_2).trim());

        // amount should be -1 for expense or 1 or income
        let mut amount = self.cad;

        if self.account_type == AccountType::Chequing && amount > 0.0 {
            amount *= -1.0;
            merchant = "".to_string();
        }

        serde_json::json!({"name": name, "merchant": merchant, "amount": amount})
    }

    pub fn from_row(row: &rusqlite::Row) -> Result<Transaction, rusqlite::Error> {
        Ok(Transaction {
            user_id: row.get(1)?,
            account_type: AccountType::from_str(&row.get::<_, String>(3)?).unwrap(),
            account_number: row.get(2)?,
            transaction_date: row.get(4)?,
            cheque_number: row.get(5)?,
            description_1: row.get(6)?,
            description_2: row.get(7)?,
            cad: row.get(8)?,
            usd: row.get(9)?,
            category: row.get(10)?,
        })
    }
}
