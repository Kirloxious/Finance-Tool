use std::collections::VecDeque;
use std::fmt::format;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::account::AccountType;
use crate::transaction::Transaction;

extern crate regex;
use regex::Regex;

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    Regex(regex::Error),
    ParseFloat(String),
    InvalidFormat(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(err) => write!(f, "IO error: {}", err),
            ParseError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            ParseError::Regex(err) => write!(f, "Regex error: {}", err),
            ParseError::ParseFloat(msg) => write!(f, "Parse float error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::Io(err)
    }
}

impl From<regex::Error> for ParseError {
    fn from(err: regex::Error) -> Self {
        ParseError::Regex(err)
    }
}

pub fn parse_csv_to_transactions(
    user_id: i64,
    path: &Path,
) -> Result<Vec<Transaction>, ParseError> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    let mut header = String::new();
    reader.read_line(&mut header)?;

    let mut transactions: Vec<Transaction> = vec![];

    for line in reader.lines() {
        let transaction = Transaction::from_rbc_csv(user_id, line?);
        transactions.push(transaction);
    }

    Ok(transactions)
}

fn _capture_groups(regex: &Regex, line: &str) -> Vec<String> {
    regex
        .captures(line)
        .unwrap()
        .iter()
        .map(|x| x.unwrap().as_str().to_string())
        .collect()
}

// parse the extracted transaction data from python script
// Format: date, description, withdrawal, deposit, balance
// XXX 00, 0000 description withdrawal deposit balance
// balance is only in the first line
// withdrawals are negatives, deposits are positives
// Sometimes the next line has a word that should be in the description
// remove lines with links (these were at the bottom of each page)
pub fn parse_extracted_transactions(
    path: &Path,
    account_number: i64,
    account_type: AccountType,
    balance: &mut f64,
    credit_limit: &mut f64,
) -> Result<Vec<Transaction>, ParseError> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);

    let mut transactions: Vec<Transaction> = vec![];

    let lines_iter = reader.lines();

    let regex =
        Regex::new(r"(?m)([a-zA-Z]{3}) ([0-9]+), ([0-9]{4}) (.*?)(( [-$]+[0-9,]+.[0-9]*)+)")
            .unwrap();

    let mut previous_line = String::new();
    let mut idx = 0;
    for line in lines_iter {
        idx += 1;
        let current_line = line?;

        // skip links
        if current_line.contains("http") {
            previous_line = current_line.clone();
            continue;
        }

        if !regex.is_match(&current_line) {
            if !transactions.is_empty()
                && regex.is_match(&previous_line)
                && current_line.split(" ").collect::<Vec<&str>>().len() < 5
            {
                transactions
                    .last_mut()
                    .unwrap()
                    .description_2
                    .push_str(current_line.clone().as_str());
            } else if idx == 6 {
                // balance and credit line
                let split_line = current_line.split(" ").collect::<Vec<_>>();
                *balance = split_line[0]
                    .replace("$", "")
                    .replace(",", "")
                    .parse::<f64>()
                    .map_err(|_| ParseError::ParseFloat(current_line.clone()))?;
                *credit_limit = split_line[1]
                    .replace("$", "")
                    .replace(",", "")
                    .parse::<f64>()
                    .map_err(|_| ParseError::ParseFloat(current_line.clone()))?;
            }
            previous_line = current_line.clone();
            continue;
        }

        // first 3 words is the date
        let mut split_line = current_line.split(' ').collect::<VecDeque<_>>();

        let date = format(format_args!(
            "{} {} {}",
            split_line.pop_front().unwrap_or_default(),
            split_line.pop_front().unwrap_or_default(),
            split_line.pop_front().unwrap_or_default(),
        ));

        let mut description = String::new();
        while !split_line.front().map(|w| w.contains("$")).unwrap_or(false) {
            description.push_str(format!("{} ", split_line.pop_front().unwrap()).as_str());
        }

        let amount = split_line.pop_front().unwrap_or_default();
        let mut amount_num = amount
            .replace("$", "")
            .replace(",", "")
            .parse::<f64>()
            .map_err(|_| ParseError::ParseFloat(amount.to_string()))?;

        if account_type == AccountType::Credit {
            amount_num *= -1.0;
        }

        let transaction = Transaction {
            user_id: 1,
            account_type,
            account_number,
            transaction_date: date,
            cheque_number: String::new(),
            description_1: description,
            description_2: String::new(),
            cad: amount_num,
            usd: 0.0,
            category: String::new(),
        };

        previous_line = current_line.clone();
        transactions.push(transaction);
    }

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use crate::calculate_hash;

    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_sample_data(file: &mut NamedTempFile) {
        writeln!(
            file,
            "Current Balance Available Balance Authorized Overdraft"
        )
        .unwrap();
        writeln!(file, "$467.17 $467.17 $0.00").unwrap();
        writeln!(file, "Date Range: Jan 1, 2020 - May 15, 2025").unwrap();

        writeln!(file, "Enter your question here Search").unwrap();
        writeln!(
            file,
            "May 12, 2025 Online Transfer to Deposit Account-7535 -$850.00 $316.17"
        )
        .unwrap();
        writeln!(
            file,
            "May 9, 2025 Online Transfer to Deposit Account-9191 -$26.64 $1,166.17"
        )
        .unwrap();
        writeln!(
            file,
            "May 8, 2025 e-Transfer - Autodeposit $25.00 $1,192.81"
        )
        .unwrap();
        writeln!(file, "Galine").unwrap(); // description_2
        writeln!(file, "C1AuCKVds4GY").unwrap(); // description_2
        writeln!(
            file,
            "May 7, 2025 Online Banking transfer - 8069 -$213.45 -$1,167.81"
        )
        .unwrap();
        writeln!(file, "May 7, 2025 Payroll Deposit $750.78").unwrap();
        writeln!(file, "CANADA").unwrap(); // description_2
    }

    #[test]
    fn test_parse_sample_transactions() {
        let mut file = NamedTempFile::new().unwrap();
        write_sample_data(&mut file);

        let path = file.path();
        let transactions = parse_extracted_transactions(
            path,
            calculate_hash(&"4325".to_string()),
            AccountType::Chequing,
            &mut 0.0,
            &mut 0.0,
        )
        .unwrap();

        assert_eq!(transactions.len(), 5);

        assert_eq!(transactions[0].transaction_date, "May 12, 2025");
        assert!(transactions[0]
            .description_1
            .contains("Deposit Account-7535"));
        assert_eq!(transactions[0].cad, -850.00);

        assert_eq!(transactions[2].transaction_date, "May 8, 2025");
        assert_eq!(transactions[2].cad, 25.00);
        assert_eq!(transactions[2].description_2, "Galine");

        assert_eq!(transactions[3].transaction_date, "May 7, 2025");
        assert!(transactions[3].description_1.contains("8069"));
        assert_eq!(transactions[3].cad, -213.45);

        assert_eq!(transactions[4].transaction_date, "May 7, 2025");
        assert_eq!(transactions[4].description_1.trim(), "Payroll Deposit");
        assert_eq!(transactions[4].description_2, "CANADA");
        assert_eq!(transactions[4].cad, 750.78);
    }

    #[test]
    fn test_parse_single_deposit_transaction() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Jan 01, 2024 Deposit Description $100.00").unwrap();

        let path = file.path();
        let result = parse_extracted_transactions(
            path,
            calculate_hash(&"123456".to_string()),
            AccountType::Chequing,
            &mut 0.0,
            &mut 0.0,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        let txn = &result[0];
        assert_eq!(txn.transaction_date, "Jan 01, 2024");
        assert_eq!(txn.description_1.trim(), "Deposit Description");
        assert_eq!(txn.description_2, "");
        assert_eq!(txn.cad, 100.0);
    }

    #[test]
    fn test_parse_transaction_with_description_2() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Jan 02, 2024 ATM Withdrawal -$50.00").unwrap();
        writeln!(file, "Extra").unwrap();

        let path = file.path();
        let result = parse_extracted_transactions(
            path,
            calculate_hash(&"123456".to_string()),
            AccountType::Savings,
            &mut 0.0,
            &mut 0.0,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        let txn = &result[0];
        assert_eq!(txn.description_2, "Extra");
    }

    #[test]
    fn test_skips_link_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "http://example.com").unwrap();
        writeln!(file, "Jan 03, 2024 Online Payment -$30.00").unwrap();

        let path = file.path();
        let result = parse_extracted_transactions(
            path,
            calculate_hash(&"123456".to_string()),
            AccountType::Chequing,
            &mut 0.0,
            &mut 0.0,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description_1.trim(), "Online Payment");
    }

    #[test]
    fn test_skip_header_lines() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "header 1").unwrap();
        writeln!(file, "header 9").unwrap();
        writeln!(file, "header header extra 20, 2024").unwrap();

        writeln!(file, "Jan 03, 2024 Online Payment -$30.00").unwrap();

        let path = file.path();
        let result = parse_extracted_transactions(
            path,
            calculate_hash(&"123456".to_string()),
            AccountType::Chequing,
            &mut 0.0,
            &mut 0.0,
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].description_1.trim(), "Online Payment");
    }

    #[test]
    fn test_multiple_transactions_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Jan 04, 2024 Salary $1000.00").unwrap();
        writeln!(file, "Jan 05, 2024 Grocery Store -$123.45").unwrap();

        let path = file.path();
        let result = parse_extracted_transactions(
            path,
            calculate_hash(&"123456".to_string()),
            AccountType::Chequing,
            &mut 0.0,
            &mut 0.0,
        )
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].description_1.trim(), "Salary");
        assert_eq!(result[0].cad, 1000.0);
        assert_eq!(result[1].description_1.trim(), "Grocery Store");
        assert_eq!(result[1].cad, -123.45);
    }
}
