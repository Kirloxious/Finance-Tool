use std::sync::{Arc, Mutex};

use crate::{account::Account, database::Database, user::User};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub user: Arc<Option<User>>,
}

impl AppState {
    pub fn new(db: Database) -> AppState {
        AppState {
            db: Arc::new(Mutex::new(db)),
            user: Arc::new(None),
        }
    }
}

pub trait App {
    fn new() -> Self;
    fn run(&self);
    fn exit(&self);
    fn login(&self, username: &str);
    fn register(&self, username: &str);
    fn logout(&self);
}

pub trait FinanceApp: App {
    fn import_transactions(&self, file_path: &str);
    fn view_transactions(&self);
    fn view_accounts(&self);
    fn balance(&self, account: &mut Account);
    fn deposit(&self, account: &mut Account);
    fn withdraw(&self, account: &mut Account);
    fn transfer(&self, account_from: &mut Account, account_to: &mut Account);
}

struct CLIApp {
    db: Box<Database>,
    user: Box<User>,
    accounts: Vec<Box<Account>>,
}

impl Default for CLIApp {
    fn default() -> Self {
        CLIApp {
            db: Box::new(Database::new("database/master.db3").unwrap()),
            user: Box::new(User::default()),
            accounts: Vec::new(),
        }
    }
}

impl App for CLIApp {
    fn new() -> Self {
        CLIApp::default()
    }

    fn run(&self) {}
    fn exit(&self) {}
    fn login(&self, username: &str) {}
    fn register(&self, username: &str) {}
    fn logout(&self) {}
}

impl FinanceApp for CLIApp {
    fn import_transactions(&self, file_path: &str) {}
    fn view_transactions(&self) {}
    fn view_accounts(&self) {}
    fn balance(&self, account: &mut Account) {}
    fn deposit(&self, account: &mut Account) {}
    fn withdraw(&self, account: &mut Account) {}
    fn transfer(&self, account_from: &mut Account, account_to: &mut Account) {}
}
