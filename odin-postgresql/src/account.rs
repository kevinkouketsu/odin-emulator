use crate::entity::account::Model as Account;
use odin_models::account::Account as AccountModel;

impl From<Account> for AccountModel {
    fn from(val: Account) -> Self {
        AccountModel {
            username: val.username,
            password: val.password,
        }
    }
}
