use crate::*;

pub const GAS_FOR_SOCIAL_GET: Gas = Gas(Gas::ONE_TERA.0 * 10);
pub const GAS_FOR_SOCIAL_SET: Gas = Gas(Gas::ONE_TERA.0 * 15);
pub const GAS_FOR_AFTER_SOCIAL_GET: Gas = Gas(Gas::ONE_TERA.0 * 50);
pub const DEPOSIT_FOR_SOCIAL_SET: Balance = 100_000_000_000_000_000_000_000;

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct GetOptions {
    pub with_block_height: Option<bool>,
    pub with_node_id: Option<bool>,
    pub return_deleted: Option<bool>,
}

#[ext_contract(ext_social)]
pub trait ExtSocial {
    fn get(self, keys: Vec<String>, options: Option<GetOptions>) -> Value;
    fn set(&mut self, data: Value);
}

#[ext_contract(ext_self)]
pub trait ExtContract {
    fn after_social_get(&mut self, #[callback_result] value: Result<Value, PromiseError>) -> usize;
}

impl Contract {
    pub fn internal_social_set(&mut self, badge: String, account_id: AccountId) {
        let mut account_data: Map<String, Value> = Map::new();
        account_data.insert(account_id.to_string(), Value::String("".to_string()));

        let mut holder_data: Map<String, Value> = Map::new();
        holder_data.insert("holder".to_string(), Value::Object(account_data));

        let mut badge_data: Map<String, Value> = Map::new();
        badge_data.insert(badge, Value::Object(holder_data));

        let mut app_data: Map<String, Value> = Map::new();
        app_data.insert("badge".to_string(), Value::Object(badge_data));

        let mut data: Map<String, Value> = Map::new();
        data.insert(env::current_account_id().to_string(), Value::Object(app_data));

        ext_social::ext(AccountId::new_unchecked(NEAR_SOCIAL_ACCOUNT_ID.to_string()))
            .with_static_gas(GAS_FOR_SOCIAL_SET)
            .with_attached_deposit(DEPOSIT_FOR_SOCIAL_SET)
            .set(
                Value::Object(data)
            );
    }
}