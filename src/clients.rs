use crate::errors::{ClientErrors, GeneralErrors};
use crate::{orders::Order, DataParser};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub struct Clients {
    pub client: BTreeMap<String, Client>,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub index: usize,
    pub name: String,
    pub dollar_balance: u32,
    pub asset_balances: Assets,
}

#[derive(Debug, Clone)]
pub struct Assets {
    pub asset: BTreeMap<String, Asset>,
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub symbol: String,
    pub balance: u32,
}

type Result<T, E> = std::result::Result<T, E>;

impl Clients {
    pub fn get_mut<T>(&mut self, client_id: T) -> Option<&mut Client>
    where
        T: Into<String>,
    {
        let client_id = &client_id.into();
        self.client.get_mut(client_id)
    }

    pub fn get_entry<T>(&mut self, client_id: T) -> Entry<String, Client>
    where
        T: Into<String>,
    {
        let client_id = client_id.into();
        self.client.entry(client_id)
    }

    pub fn get<T>(&self, client_id: T) -> Option<Client>
    where
        T: Into<String>,
    {
        let client_id = &client_id.into();
        Some(self.client.get(client_id)?.clone())
    }
}


// Preparing clients for recording.
impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}\t", self.name))?;
        f.write_fmt(format_args!("{}\t", self.dollar_balance))?;
        f.write_fmt(format_args!(
            "{}\t",
            self.asset_balances.get("A").unwrap().balance
        ))?;
        f.write_fmt(format_args!(
            "{}\t",
            self.asset_balances.get("B").unwrap().balance
        ))?;
        f.write_fmt(format_args!(
            "{}\t",
            self.asset_balances.get("C").unwrap().balance
        ))?;
        f.write_fmt(format_args!(
            "{}\n",
            self.asset_balances.get("D").unwrap().balance
        ))
    }
}

impl Client {
    // Purchase Error Checking.
    pub fn check_buy_error(&self, cache_order: &Order, order: &Order) -> Result<(), GeneralErrors> {
        if self.dollar_balance < cache_order.order_price * order.value {
            return Err(GeneralErrors::NotEnaughDollars);
        }
        Ok(())
    }
    // Sales Error Checking.
    pub fn check_sell_error(&self, order: &Order) -> Result<(), GeneralErrors> {
        let asset_balance = self
            .asset_balances
            .get(&order.asset)
            .ok_or(GeneralErrors::GetClientError)?;
        if asset_balance.balance < order.value {
            return Err(GeneralErrors::NotEnaughAsset);
        }
        Ok(())
    }

    // There are no balance checks in the two functions 
    // described below, because the checks are performed in the functions above.
    // These functions are used to reduce repetitive code in main.rs.
    pub fn buy(&mut self, cache_order: &Order, order_value: u32) {
        let mut asset_balance = self.asset_balances.get_mut(&cache_order.asset).unwrap();
        self.dollar_balance -= cache_order.order_price * order_value;
        asset_balance.balance += order_value;
    }

    pub fn sell(&mut self, cache_order: &Order, order_value: u32) {
        let mut asset_balance = self.asset_balances.get_mut(&cache_order.asset).unwrap();
        asset_balance.balance -= order_value;
        self.dollar_balance += cache_order.order_price * order_value;
    }
}


impl DataParser for Clients {
    type Item = Client;
    type Err = ClientErrors;

    fn new() -> Clients {
        Clients {
            client: BTreeMap::new(),
        }
    }

    fn insert(&mut self, index: usize, mut data: Self::Item) {
        data.index = index;
        self.client.insert(data.name.clone(), data);
    }

    fn parse(line: &str) -> Result<Self::Item, Self::Err> {
        Self::Item::from_str(line)
    }

    fn remove(&mut self, clients: Vec<Self::Item>) {
        for client in clients {
            self.client.remove(&client.name);
        }
    }
}

// FromStr impl for Order struct.
impl FromStr for Client {
    type Err = ClientErrors;

    // Converting a String to an Order Structure
    fn from_str(s: &str) -> Result<Self, ClientErrors> {
        use ClientErrors::*;
        let vals: Vec<&str> = s.split_whitespace().collect();

        if vals.len() < 6 {
            return Err(ClientErrors::ParseInsufficentInput);
        }

        let index: usize = 0;
        let name = vals[0].parse::<String>().map_err(|_| ParseClientIdError)?;
        let dollar_balance = vals[1]
            .parse::<u32>()
            .map_err(|_| ParseDollarBalanceError)?;
        let a_balance = Asset {
            symbol: "A".to_string(),
            balance: vals[2]
                .parse::<u32>()
                .map_err(|_| ParseAssetBalancesError)?,
        };
        let b_balance = Asset {
            symbol: "B".to_string(),
            balance: vals[3]
                .parse::<u32>()
                .map_err(|_| ParseAssetBalancesError)?,
        };
        let c_balance = Asset {
            symbol: "C".to_string(),
            balance: vals[4]
                .parse::<u32>()
                .map_err(|_| ParseAssetBalancesError)?,
        };
        let d_balance = Asset {
            symbol: "D".to_string(),
            balance: vals[5]
                .parse::<u32>()
                .map_err(|_| ParseAssetBalancesError)?,
        };

        let mut asset_balances: Assets = Assets {
            asset: BTreeMap::new(),
        };

        asset_balances
            .asset
            .insert(a_balance.symbol.clone(), a_balance);
        asset_balances
            .asset
            .insert(b_balance.symbol.clone(), b_balance);
        asset_balances
            .asset
            .insert(c_balance.symbol.clone(), c_balance);
        asset_balances
            .asset
            .insert(d_balance.symbol.clone(), d_balance);

        Ok(Client {
            index,
            name,
            dollar_balance,
            asset_balances,
        })
    }
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            asset: BTreeMap::new(),
        }
    }

    pub fn get_mut<T>(&mut self, asset_id: T) -> Option<&mut Asset>
    where
        T: Into<String>,
    {
        let asset_id = &asset_id.into();
        self.asset.get_mut(asset_id)
    }

    pub fn get<T>(&self, asset_id: T) -> Option<Asset>
    where
        T: Into<String>,
    {
        let asset_id = &asset_id.into();
        Some(self.asset.get(asset_id)?.clone())
    }
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let base_string = "C5    11    4    63    124    33";
        let struct_exemplar = Client::from_str(base_string).unwrap();
        let a_balance = struct_exemplar.asset_balances.get("A").unwrap();
        let b_balance = struct_exemplar.asset_balances.get("B").unwrap();
        let c_balance = struct_exemplar.asset_balances.get("C").unwrap();
        let d_balance = struct_exemplar.asset_balances.get("D").unwrap();
        assert_eq!(struct_exemplar.name, "C5");
        assert_eq!(struct_exemplar.dollar_balance, 11);
        assert_eq!(a_balance.balance, 4);
        assert_eq!(b_balance.balance, 63);
        assert_eq!(c_balance.balance, 124);
        assert_eq!(d_balance.balance, 33);
    }

    #[test]
    fn test_parse_insufficent_error_from_str() {
        let base_string = "24    11    4    63    124    ";
        let actual_error = Client::from_str(base_string).unwrap_err();
        let expected_error = ClientErrors::ParseInsufficentInput;
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn test_parse_client_dollar_balance_error_from_str() {
        let base_string = "24    B    4    63    124    33";
        let actual_error = Client::from_str(base_string).unwrap_err();
        let expected_error = ClientErrors::ParseDollarBalanceError;
        assert_eq!(actual_error, expected_error)
    }

    #[test]
    fn test_parse_client_asset_error_from_str() {
        let base_string = "24    11    B    63    124    33";
        let actual_error = Client::from_str(base_string).unwrap_err();
        let expected_error = ClientErrors::ParseAssetBalancesError;
        assert_eq!(actual_error, expected_error)
    }
}
