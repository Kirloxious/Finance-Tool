use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{http::StatusCode, Json, Router};
use finance_tool::account::{AccountType, BankAccount};
use finance_tool::app::{App, AppState};
use finance_tool::catergorization::catergorize_transactions;
use finance_tool::database::Database;
use finance_tool::parser;
use finance_tool::user::User;
use rusqlite::Result;
use tokio::task;

// TODO: Handle errors
// Reminder: Send errors upstream to deal with in main routine

async fn _test_setup() -> Result<()> {
    let db = Database::new("database/master.db3")?;
    // db._execute_schema()?;

    db.reset_values()?;
    let user_name = String::from("Alex");

    let user = User::new(user_name);

    db.insert_user(&user)?;

    let credit_path = Path::new("data/credit_data.txt");
    let savings_path = Path::new("data/savings_data.txt");
    let chequing_path = Path::new("data/chequing_data.txt");

    let mut chequing_balance = 0.0;
    let mut savings_balance = 0.0;
    let mut credit_balance = 0.0;
    let mut credit_limit = 0.0;

    let mut transactions =
        parser::parse_csv_to_transactions(user.id, Path::new("data/csv48685.csv")).unwrap();

    for transaction in &mut transactions {
        let acc = transaction.extract_account().unwrap();
        if !db.account_exists(acc.account_number()).unwrap_or(true) {
            db.insert_account(&acc)?;
        }
        transaction.user_id = user.id;
    }

    let categories = catergorize_transactions(&transactions)
        .await
        .map_err(|e| println!("{}", e))
        .unwrap();

    for (tx, cat) in transactions.iter_mut().zip(categories.into_iter()) {
        tx.category = cat;
    }

    db.batch_insert_transactions(&transactions)?;

    let mut credit_transactions = match parser::parse_extracted_transactions(
        credit_path,
        db.get_account_number_by_type(user.id, &AccountType::Credit)?,
        AccountType::Credit,
        &mut credit_balance,
        &mut credit_limit,
    ) {
        Ok(t) => t,
        Err(e) => panic!("{}", e),
    };
    let savings_transactions = match parser::parse_extracted_transactions(
        savings_path,
        db.get_account_number_by_type(user.id, &AccountType::Savings)?,
        AccountType::Savings,
        &mut savings_balance,
        &mut 0.0,
    ) {
        Ok(t) => t,
        Err(e) => panic!("{}", e),
    };
    let chequing_transactions = match parser::parse_extracted_transactions(
        chequing_path,
        db.get_account_number_by_type(user.id, &AccountType::Chequing)?,
        AccountType::Chequing,
        &mut chequing_balance,
        &mut 0.0,
    ) {
        Ok(t) => t,
        Err(e) => panic!("{}", e),
    };

    credit_transactions.extend(savings_transactions);
    credit_transactions.extend(chequing_transactions);

    for transaction in &mut credit_transactions {
        transaction.user_id = user.id;
    }

    let categories = catergorize_transactions(&credit_transactions)
        .await
        .map_err(|e| println!("{}", e))
        .unwrap();

    for (tx, cat) in credit_transactions.iter_mut().zip(categories.into_iter()) {
        tx.category = cat;
    }

    db.batch_insert_transactions(&credit_transactions)?;

    let accounts = db.get_accounts_by_user(user.id).unwrap();

    for mut account in accounts {
        match account.account_type() {
            AccountType::Credit => {
                println!("Credit balance: {}", credit_balance);
                println!("Credit limit: {}", credit_limit);

                account.set_balance(credit_balance);
                account.set_credit_limit(credit_limit);
            }
            AccountType::Savings => {
                println!("Savings balance: {}", savings_balance);

                account.set_balance(savings_balance);
                account.set_credit_limit(0.0);
            }
            AccountType::Chequing => {
                println!("Chequing balance: {}", chequing_balance);

                account.set_balance(chequing_balance);
                account.set_credit_limit(0.0);
            }
            AccountType::Unknown => println!("Unknown account type: {}", account.account_type()),
        }
        db.update_account(&account)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let db = Database::new("database/master.db3").unwrap();

    let state = AppState::new(db);

    let app = Router::new()
        .route("/", get(root))
        .route("/users/{username}", get(get_user))
        .with_state(state);

    let listerner = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listening on port 3000");
    axum::serve(listerner, app).await.unwrap();
}

async fn get_user(
    axum::extract::Path(username): axum::extract::Path<String>,
    State(mut state): State<AppState>,
) -> (StatusCode, Json<User>) {
    let conn = state.db.clone();
    let user = task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        conn.get_user_by_name(&username).unwrap()
    })
    .await
    .unwrap();

    state.user = Arc::new(Some(user.clone()));

    (StatusCode::OK, Json(user))
}

async fn root() -> &'static str {
    "Hello, world!"
}
