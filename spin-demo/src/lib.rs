use spin_sdk::http::{Response, Request};
use spin_sdk::{http_component, pg::{self}};
use anyhow::{anyhow, Result};
use std::error::Error;
use std::collections::HashMap;
use chrono::prelude::*;

// defined in spin.toml
const DB_URL_ENV: &str = "DB_URL";

#[derive(serde::Deserialize, Debug)]
struct Customer {
    name: String,
    email: String,
    phone: String,
    address: String,
    city: String,
    pin: String,
}

#[derive(serde::Deserialize, Debug)]
struct Cart {
    id: i32,
    name: String,
    image: String,
    price: i32,
    quantity: i32,
}

#[derive(serde::Deserialize, Debug)]
struct Order {
    customer: Customer,
    checkout: Vec<Cart>,
    #[serde(rename(deserialize = "orderTotal"))]
    order_total: String,
    #[serde(rename(deserialize = "orderState"))]
    order_state: OrderState
}


#[derive(serde::Deserialize, PartialEq, Debug)]
enum OrderState {
    Checkout,
    Pending,
    Processing,
    Canceled,
    Completed,
}

#[http_component]
async fn order_request(req: Request) -> Result<Response, Box<dyn Error>> {
    let start_time: DateTime<Utc> = Utc::now();
    let mut ordered_products: HashMap<String, i32> = HashMap::new();

    match serde_json::from_slice::<Order>(req.body().as_ref()) {
        Ok(order) => {
            println!("Incoming order request, order state: {:?}", order.order_state);
            let result = handle_order(order, start_time, &mut ordered_products).expect("Handling order failed");
            let end_time: DateTime<Utc> = Utc::now();
            println!("Order state: {:?}", result);
            Ok(Response::builder()
                .status(200)
                .header("content-type", "application/json")
                .header("Access-Control-Allow-Origin", "http://localhost:5173")
                .header("Access-Control-Allow-Methods", "POST")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .body(format!("Execution time: {}. Order: {:?}", end_time - start_time, result))
                .build())
        }
        Err(e) => {
            eprintln!("Failed to parse request body: {:?}", e);
            Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .header("Access-Control-Allow-Origin", "http://localhost:5173")
                .header("Access-Control-Allow-Methods", "POST")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .body(format!("Invalid request body: {}", e))
                .build())
        }
    }
}

fn handle_order(
    order: Order,
    start_time: DateTime<Utc>,
    ordered_products: &mut HashMap<String, i32>
) -> Result<OrderState> {
    let address = std::env::var(DB_URL_ENV)?;
    let conn = pg::Connection::open(&address)?;
    let mut current_order_state: OrderState = OrderState::Pending;

    for article in &order.checkout {
        let current_quantity =
            get_product_quantity(&conn, article).expect("Product quantity query failed") - article.quantity;
        if current_quantity <= 0 {
            current_order_state = OrderState::Canceled;
            println!("Insufficient stock for product: {:?}", article.name)
        } else {
            let sql = format!("UPDATE products.\"product_details\" \
            SET quantity = '{}' WHERE name = '{}'", current_quantity, article.name);
            conn.query(&sql, &[])?;
            println!("Updated product stock for product: {}", article.name);

            ordered_products.insert(article.name.clone(), article.quantity);
            current_order_state = OrderState::Processing;
        }
    }

    if current_order_state == OrderState::Processing {
        insert_customer_data(&conn, &order, start_time).expect("Customer data insert failed");
        current_order_state = insert_order_detail_data(&conn, &order, ordered_products, start_time).expect("Order data insert failed");
    }

    Ok(current_order_state)
}

fn get_product_quantity(conn: &pg::Connection, article: &Cart) -> Result<i32> {
    // Welcome to SQL-Injection its cool, its fun for everybody!!!
    let sql =
        format!("SELECT quantity FROM products.\"product_details\" \
        WHERE name = '{}' AND article_number = '{}'", article.name, article.id);
    // ParameterValue is not working - fix later (maybe)
    //let params = vec![ParameterValue::Str(article.name)];
    let rowset = conn.query(&sql, &[])?;

    match rowset.rows.first() {
        None => Ok(0),
        Some(row) => match row.first() {
            None => Ok(0),
            Some(pg::DbValue::Int32(i)) => Ok(*i),
            Some(other) => Err(anyhow!(
                "Unexpected error while fetching article quantity from database: {:?}", other)
            ),
        },
    }
}

fn get_customer_id(conn: &pg::Connection, customer: &Customer) -> Result<i32> {
    let sql =
        format!("SELECT id FROM products.\"customers\" \
        WHERE name = '{}' AND email = '{}'", customer.name, customer.email);
    let rowset = conn.query(&sql, &[])?;

    match rowset.rows.first() {
        None => Ok(0),
        Some(row) => match row.first() {
            None => Ok(0),
            Some(pg::DbValue::Int32(i)) => Ok(*i),
            Some(other) => Err(anyhow!(
                "Unexpected error while fetching customer id from database: {:?}", other)
            ),
        },
    }
}

fn get_order_id(conn: &pg::Connection, start_time: DateTime<Utc>, customer_id: i32) -> Result<i32> {
    let sql = format!("SELECT id FROM products.\"order_details\" WHERE customer_id = '{}' AND timestamp_created = '{}'", customer_id, start_time);
    let rowset = conn.query(&sql, &[])?;

    match rowset.rows.first() {
        None => Ok(0),
        Some(row) => match row.first() {
            None => Ok(0),
            Some(pg::DbValue::Int32(i)) => Ok(*i),
            Some(other) => Err(anyhow!(
                "Unexpected error while fetching order id from database: {:?}", other)
            ),
        },
    }
}

fn insert_customer_data(
    conn: &pg::Connection,
    order: &Order,
    start_time: DateTime<Utc>
) -> Result<u64> {
    let sql = format!("INSERT INTO products.\"customers\" \
    (name, email, phone, address, city, pin, last_ordered) \
    VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}') \
    ON CONFLICT (name, email) DO UPDATE SET last_ordered = EXCLUDED.last_ordered",
                      order.customer.name,
                      order.customer.email,
                      order.customer.phone,
                      order.customer.address,
                      order.customer.city,
                      order.customer.pin,
                      start_time
    );
    let rows_affected = conn.execute(&sql, &[])?;
    println!("Updated customer data for customer: {:?}", order.customer.name);
    Ok(rows_affected.try_into().unwrap())
}

fn insert_order_detail_data(
    conn: &pg::Connection,
    order: &Order,
    ordered_products: &mut HashMap<String, i32>,
    start_time: DateTime<Utc>
) -> Result<OrderState> {
    let sql = format!("INSERT INTO products.\"order_details\" \
    (timestamp_created, total_products, total_price, customer_id, order_state) VALUES ('{}', '{}', '{}', '{}', '{:?}')",
                      start_time,
                      serde_json::to_string(&ordered_products).unwrap(),
                      order.order_total,
                      get_customer_id(&conn, &order.customer).unwrap(),
                      OrderState::Completed,
    );
 
    conn.execute(&sql, &[])?;


    for article in &order.checkout {
        let quantity = ordered_products.get(&article.name).unwrap();
        insert_order_item_data(conn, article, order, *quantity, start_time).expect("failed insert order items.");
    }
    
    println!("Stored order data for: {:?}", serde_json::to_string(&ordered_products).unwrap());
    Ok(OrderState::Completed)
}

fn insert_order_item_data(conn: &pg::Connection, article: &Cart, order: &Order, order_quantity: i32, start_time: DateTime<Utc>) -> Result<u64> {
    let customer_id = get_customer_id(conn, &order.customer).unwrap();
    let order_id = get_order_id(conn, start_time, customer_id).unwrap();
    println!("CUST_ID = {}, ORDER_ID = {}", customer_id, order_id);
    let sql = format!("INSERT INTO products.\"order_items\" (order_id, product_id, order_quantity, timestamp_created) VALUES ('{}', '{}', '{}', '{}')", order_id, article.id, order_quantity, start_time);
    
    let rows_affected = conn.execute(&sql, &[])?;
    println!("Updated order details data for article: {:?}", article.name);
    Ok(rows_affected.try_into().unwrap())
}
