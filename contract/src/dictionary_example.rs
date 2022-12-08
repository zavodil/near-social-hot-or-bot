pub const DICTIONARY_VALUES: u32 = 20;

pub fn get_hot(index: usize) -> String {
    vec![
        "hash1",
        "hash2",
    ].get(index).expect("ERR_NO_VALUE").to_string()
}

pub fn get_bot(index: usize) -> String {
    vec![
        "hash3",
        "hash4",
    ].get(index).expect("ERR_NO_VALUE").to_string()
}
