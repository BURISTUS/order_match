use std::{fs::File, io::Write};
use trade_match::{
    clients::Clients,
    config::get_config,
    errors::GeneralErrors,
    orders::OrderType,
    orders::{Order, Orders},
    read_file, DataParser,
};

fn main() -> Result<(), GeneralErrors> {
    let file_path = get_config().map_err(|_| GeneralErrors::GetConfigError)?;
    let mut clients: Clients =
        read_file(file_path.clients).map_err(|_| GeneralErrors::ReadFileError)?;
    let orders: Orders = read_file(file_path.orders).map_err(|_| GeneralErrors::ReadFileError)?;
    match_orders(&mut clients, orders);
    write_file(clients);
    Ok(())
}

// A function that checks orders by operation and calls a method depending on it.
fn match_orders(clients: &mut Clients, orders: Orders) {
    let mut buy_orders = Orders::new();
    let mut sell_orders = Orders::new();

    for mut order in orders.order {
        match order.1.operation {
            OrderType::Buy => {
                let (completed_orders, updated_orders) = buy_assets(&sell_orders, clients, &mut order.1);
                if order.1.value > 0 {
                    buy_orders.insert(order.0, order.1);
                }
                if !completed_orders.is_empty() {
                    sell_orders.remove(completed_orders);
                }
                if !updated_orders.is_empty() {
                    sell_orders.update_orders(updated_orders);
                }
            }
            OrderType::Sell => {
                let (completed_orders, updated_orders) = sell_assets(&buy_orders, clients, &mut order.1);
                if order.1.value > 0 {
                    sell_orders.insert(order.0, order.1);
                }
                if !completed_orders.is_empty() {
                    buy_orders.remove(completed_orders);
                }
                if !updated_orders.is_empty() {
                    buy_orders.update_orders(updated_orders);
                }
            }
            _ => continue,
        }
    }
}

fn buy_assets(sell_orders: &Orders, clients: &mut Clients, order: &mut Order) -> (Vec<Order>, Vec<Order>) {
    let mut sell_asset_orders: Vec<Order> = Vec::new();
    let mut completed_orders: Vec<Order> = Vec::new();
    let mut updated_orders: Vec<Order> = Vec::new();
    
    // Order vector formation.
    for ord in &sell_orders.order {
        if ord.1.client_name != order.client_name
            && order.asset == ord.1.asset
            && order.order_price >= ord.1.order_price
        {
            sell_asset_orders.push(ord.1.clone());
        }
    }

    sell_asset_orders.sort_by(|a, b| a.order_price.cmp(&b.order_price));

    for mut sell_order in sell_asset_orders {

        if order.value == 0 {
            break;
        }

        // Check for balance errors.
        let buyer_err = clients.get(&order.client_name).and_then(|c| {
            if c.check_buy_error(&sell_order, order).is_err() {
                None
            } else {
                Some(true)
            }
        });

        let seller_err = clients.get(&sell_order.client_name).and_then(|c| {
            if c.check_sell_error(order).is_err() {
                None
            } else {
                Some(true)
            }
        });

        // Go to the next iteration in case of an error.
        if buyer_err.is_none() || seller_err.is_none() {
            continue;
        }


        clients
            .get_entry(order.client_name.clone())
            .and_modify(|c| {

                if order.value > sell_order.value {
                    c.buy(&sell_order, sell_order.value);
                } else {
                    c.buy(&sell_order, order.value);
                }

            });

        clients
            .get_entry(sell_order.client_name.clone())
            .and_modify(|c| {

                if order.value > sell_order.value {
                    c.sell(&sell_order, sell_order.value);
                
                    order.value -= sell_order.value;
                    completed_orders.push(sell_order.clone());
                } else {
                    c.sell(&sell_order, order.value);
                   
                    sell_order.value -= order.value;
                    updated_orders.push(sell_order.clone());
                    order.value = 0;
                }

            });

        
    }
    (completed_orders, updated_orders)
}

fn sell_assets(buy_orders: &Orders, clients: &mut Clients, order: &mut Order) -> (Vec<Order>, Vec<Order>) {
    let mut buy_asset_orders: Vec<Order> = Vec::new();
    let mut completed_orders: Vec<Order> = Vec::new();
    let mut updated_orders: Vec<Order> = Vec::new();

    // Order vector formation.
    for ord in &buy_orders.order {
        if ord.1.client_name != order.client_name
            && order.asset == ord.1.asset
            && order.order_price < ord.1.order_price
        {
            buy_asset_orders.push(ord.1.clone());
        }
    }

    buy_asset_orders.sort_by(|a, b| a.order_price.cmp(&b.order_price));
    
    for mut buy_order in buy_asset_orders {

        if order.value == 0 {
            break;
        }

        // Check for balance errors.
        let buyer_err = clients.get(&buy_order.client_name).and_then(|c| {
            if c.check_buy_error(&buy_order, order).is_err() {
                None
            } else {
                Some(true)
            }
        });

        let seller_err = clients.get(&order.client_name).and_then(|c| {
            if c.check_sell_error(order).is_err() {
                None
            } else {
                Some(true)
            }
        });
        
        // Go to the next iteration in case of an error.
        if buyer_err.is_none() || seller_err.is_none() {
            continue;
        }

        clients
            .get_entry(order.client_name.clone())
            .and_modify(|c| {

                if order.value >= buy_order.value {
                    c.sell(&buy_order, buy_order.value);
                } else {
                    c.sell(&buy_order, order.value);
                }

            });

        clients
            .get_entry(buy_order.client_name.clone())
            .and_modify(|c| {
                
                if order.value > buy_order.value {
                    c.buy(&buy_order, buy_order.value);

                    order.value -= buy_order.value;
                    completed_orders.push(buy_order.clone());
                } else {
                    c.buy(&buy_order, order.value);

                    buy_order.value -= order.value;
                    updated_orders.push(buy_order.clone());
                    order.value = 0;
                }

            });
        
    }
    (completed_orders, updated_orders)
}

fn write_file(clients: Clients) {
    let mut file = File::create("Result.txt").unwrap();
    for (_, client) in clients.client {
        write!(file, "{}", client).unwrap();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use trade_match::{
        clients::{Asset, Assets, Client},
        orders::{Order, Orders},
    };

    #[test]
    fn test_buy() {
        let mut buyer_asset = Assets::new();
        let client1_asset = Asset {
            symbol: "A".to_string(),
            balance: 25,
        };
        buyer_asset
            .asset
            .insert(client1_asset.symbol.clone(), client1_asset);

        let mut seller_asset = Assets::new();
        let client2_asset = Asset {
            symbol: "A".to_string(),
            balance: 25,
        };
        seller_asset
            .asset
            .insert(client2_asset.symbol.clone(), client2_asset);

        let mut clients = Clients::new();
        let buy_client = Client {
            index: 0,
            name: "C2".to_string(),
            dollar_balance: 1000,
            asset_balances: buyer_asset,
        };

        let sell_client = Client {
            index: 0,
            name: "C3".to_string(),
            dollar_balance: 1000,
            asset_balances: seller_asset,
        };
        clients.insert(1, buy_client);
        clients.insert(2, sell_client);

        let mut orders: Orders = Orders::new();

        let sell_order_1 = Order {
            index: 0,
            client_name: "C2".to_string(),
            operation: OrderType::Sell,
            asset: "A".to_string(),
            order_price: 8,
            value: 4,
        };

        let mut buy_order_1 = Order {
            index: 2,
            client_name: "C3".to_string(),
            operation: OrderType::Buy,
            asset: "A".to_string(),
            order_price: 10,
            value: 6,
        };

        orders.insert(sell_order_1.index, sell_order_1);

        buy_assets(&orders, &mut clients, &mut buy_order_1);

        assert_eq!(clients.get("C2".to_string()).unwrap().dollar_balance, 1032);
        assert_eq!(
            clients
                .get("C2".to_string())
                .unwrap()
                .asset_balances
                .get("A".to_string())
                .unwrap()
                .balance,
            21
        );
        assert_eq!(clients.get("C3".to_string()).unwrap().dollar_balance, 968);
        assert_eq!(
            clients
                .get("C3".to_string())
                .unwrap()
                .asset_balances
                .get("A".to_string())
                .unwrap()
                .balance,
            29
        );
    }

    #[test]
    fn test_sell() {
        let mut buyer_asset = Assets::new();
        let client1_asset = Asset {
            symbol: "A".to_string(),
            balance: 25,
        };
        buyer_asset
            .asset
            .insert(client1_asset.symbol.clone(), client1_asset);

        let mut seller_asset = Assets::new();
        let client2_asset = Asset {
            symbol: "A".to_string(),
            balance: 25,
        };
        seller_asset
            .asset
            .insert(client2_asset.symbol.clone(), client2_asset);

        let mut clients = Clients::new();
        let buy_client = Client {
            index: 0,
            name: "C2".to_string(),
            dollar_balance: 1000,
            asset_balances: buyer_asset,
        };

        let sell_client = Client {
            index: 0,
            name: "C3".to_string(),
            dollar_balance: 1000,
            asset_balances: seller_asset,
        };
        clients.insert(1, buy_client);
        clients.insert(2, sell_client);

        let mut orders: Orders = Orders::new();

        let mut sell_order_1 = Order {
            index: 0,
            client_name: "C2".to_string(),
            operation: OrderType::Sell,
            asset: "A".to_string(),
            order_price: 8,
            value: 4,
        };

        let buy_order_1 = Order {
            index: 2,
            client_name: "C3".to_string(),
            operation: OrderType::Buy,
            asset: "A".to_string(),
            order_price: 10,
            value: 6,
        };

        orders.insert(buy_order_1.index, buy_order_1);

        sell_assets(&orders, &mut clients, &mut sell_order_1);

        assert_eq!(clients.get("C2".to_string()).unwrap().dollar_balance, 1040);
        assert_eq!(
            clients
                .get("C2".to_string())
                .unwrap()
                .asset_balances
                .get("A".to_string())
                .unwrap()
                .balance,
            21
        );
        assert_eq!(clients.get("C3".to_string()).unwrap().dollar_balance, 960);
        assert_eq!(
            clients
                .get("C3".to_string())
                .unwrap()
                .asset_balances
                .get("A".to_string())
                .unwrap()
                .balance,
            29
        );
    }
}