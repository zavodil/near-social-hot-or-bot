use near_sdk::{
    near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Gas, env, ext_contract, log, PromiseError, Promise, PromiseOrValue, require, Balance,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, LazyOption},
    serde::{Deserialize, Serialize},
};
use sha2::{Sha256, Digest};
use near_sdk::serde_json::{Map, Value};
use near_contract_standards::non_fungible_token::{NonFungibleToken, Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::{NFT_METADATA_SPEC, NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata};

mod nft;
mod dictionary;
mod social;

use crate::nft::*;
use crate::social::*;
use crate::dictionary::*;

//const NEAR_SOCIAL_ACCOUNT_ID: &str = "v1.social08.testnet";
//const NEAR_SOCIAL_APP_NAME: &str = "HotOrBotTestnet";
const NEAR_SOCIAL_ACCOUNT_ID: &str = "social.near";
const NEAR_SOCIAL_APP_NAME: &str = "HotOrBot";
const NEAR_SOCIAL_WINNER_BADGE: &str = "winner";

const MAX_TURNS: usize = 4;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    PlayersScore,
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    players_score: LookupMap<AccountId, usize>,
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    next_token_id: u64,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            players_score: LookupMap::new(StorageKey::PlayersScore),
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                env::current_account_id(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&internal_get_metadata())),
            next_token_id: 0,
        }
    }

    // return a challenge for a given turn for specific account_id
    pub fn get_turn(&self, account_id: AccountId, turn: usize) -> Vec<String> {
        let turns = self.get_turns(account_id);
        assert!(turns.len() >= turn, "ERR_WRONG_TURN");

        let item: usize = turns[turn] as usize;

        let index = get_binary_random();
        let mut result = vec!["".to_string(); 2];
        result[index] = get_bot(item);
        result[1 - index] = get_hot(item);

        result
    }

    // return array of indexes for challenges for a given account_id. Every index < DICTIONARY_VALUES
    pub fn get_turns(&self, account_id: AccountId) -> Vec<u32> {
        let hash = format!("{:x}", Sha256::digest(account_id.as_bytes()));

        hash.chars().fold(vec![], |mut result, c| {
            let value = c as u32 % DICTIONARY_VALUES;
            if !result.contains(&value) {
                result.push(value);
            }
            result
        })
    }

    // finalize game. Read data from social, verify, calculate score and create NFT & Social badge for winners
    #[payable]
    pub fn nft_mint(&mut self, receiver_id: AccountId) -> PromiseOrValue<usize> {
        let account_id = env::predecessor_account_id();
        require!(receiver_id == account_id, "Illegal receiver");
        require!(self.players_score.get(&account_id).is_none(), "Already finalized");

        let get_request = format!("{}/{}/**", account_id, NEAR_SOCIAL_APP_NAME);

        ext_social::ext(AccountId::new_unchecked(NEAR_SOCIAL_ACCOUNT_ID.to_string()))
            .with_static_gas(GAS_FOR_SOCIAL_GET)
            .get(
                vec![get_request],
                None,
            )
            .then(
                ext_self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_AFTER_SOCIAL_GET)
                    .with_attached_deposit(env::attached_deposit())
                    .after_social_get()
            ).into()
    }

    #[payable]
    #[private]
    pub fn after_social_get(
        &mut self,
        #[callback_result] value: Result<Value, PromiseError>,
    ) -> usize {
        let mut score: usize = 0;
        if let Ok(mut value) = value {
            let data = value.as_object_mut().expect("Data is not a JSON object");
            for (account_id, value) in data {
                let account_id = AccountId::new_unchecked(account_id.to_owned());
                let turns = self.get_turns(account_id.clone());

                for (turn_index, turn_data) in value.get(NEAR_SOCIAL_APP_NAME.to_string()).expect("Missing data").as_object().expect("Missing turns") {
                    let turn_index = turn_index.to_owned().parse::<usize>().unwrap();
                    require!(turn_index < MAX_TURNS, "Illegal turn index");
                    for (key, value) in turn_data.as_object().unwrap() {
                        let value = value.as_str().unwrap();
                        if key == "bot" {
                            let turn_key = turns[turn_index] as usize;
                            if get_bot(turn_key) == value {
                                score += 1;
                            }
                        }
                    }
                }

                self.players_score.insert(&account_id, &score);

                if score == MAX_TURNS {
                    self.internal_mint(&account_id);
                    self.internal_social_set(NEAR_SOCIAL_WINNER_BADGE.to_string(), account_id);
                    log!("You win!");
                }
                else{
                    log!("You didn't win. Deposit for NFT storage reverted");
                    Promise::new(account_id).transfer(env::attached_deposit());
                }
            }
        }

        score
    }

    pub fn get_score(&self, account_id: AccountId) -> Option<usize> {
        self.players_score.get(&account_id)
    }

    // legacy method to reward first winners
    #[private]
    pub fn set_winner(&mut self, account_id: AccountId) {
        self.internal_social_set(NEAR_SOCIAL_WINNER_BADGE.to_string(), account_id);
    }
}

pub fn get_binary_random() -> usize {
    let random_seed = env::random_seed();
    (random_seed[0] % 2) as usize
}
