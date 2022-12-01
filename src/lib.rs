use std::{
    error, fs,
    io::{self, BufRead},
    str::FromStr,
};

pub mod clients;
pub mod config;
pub mod errors;
pub mod orders;

pub type Volume = u32;
pub type Price = u32;

// Trait for parsing input data
pub trait DataParser {
    type Item;
    type Err;

    fn new() -> Self;
    fn insert(&mut self, index: usize, data: Self::Item);
    fn parse(line: &str) -> Result<Self::Item, Self::Err>;
    fn remove(&mut self, orders: Vec<Self::Item>);
}

pub fn read_file<T>(file_path: String) -> Result<T, &'static dyn error::Error>
where
    T: DataParser,
    T::Item: FromStr,
    T::Err: std::fmt::Debug,
{
    let file = fs::File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);
    let mut data = T::new();
    let mut index: usize = 1;
    for (_, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let struct_exemplar = T::parse(line.as_str()).unwrap();
        data.insert(index, struct_exemplar);
        index += 1;
    }

    Ok(data)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::clients::Clients;

    #[test]
    fn test_read_file() {
        let file = "./Clients.txt";
        let clients: Clients =
            read_file(file.to_string()).expect("Failed to read clients from file");
        let client = clients.get("C1").unwrap();
        assert_eq!(client.name, "C1");
        assert_eq!(client.dollar_balance, 1000);
        assert_eq!(client.asset_balances.get("A").unwrap().balance, 130);
        assert_eq!(client.asset_balances.get("B").unwrap().balance, 240);
        assert_eq!(client.asset_balances.get("C").unwrap().balance, 760);
        assert_eq!(client.asset_balances.get("D").unwrap().balance, 320);
    }
}