use crate::structures::{Delivery, Item, Order, Payment};
use axum::Json;
use std::error::Error;
use log::{debug, error, info};
use tokio_postgres::{Client, Connection, NoTls};

//Структура с основными данными для подключения к бд
#[derive(Clone)]
pub struct db_model {
    db_type: String,
    host: String,
    user: String,
    password: String,
    db_name: String,
    port: String,
}

impl db_model {
    //Создание нового экземпляра структуры выполняется при помощи данного метода
    pub fn new(
        db_type: String,
        host: String,
        user: String,
        password: String,
        db_name: String,
        port: String,
    ) -> Self {
        info!("Setting parameters for connecting to data base");
        db_model {
            db_type: db_type,
            host: host,
            user: user,
            password: password,
            db_name: db_name,
            port: port,
        }
    }
    //Подключение предоставление пользователя для совершения запросов в бд
    pub async fn init(&self) -> Client {
        //Формирование url для подключения к бд
        let url = format!(
            "{}://{}:{}@{}:{}/{}",
            &self.db_type, &self.user, &self.password, &self.host, &self.port, &self.db_name
        );
        debug!("Connect to data base on {}", url);
        let (client, connection) = tokio_postgres::connect(&url, NoTls)
            .await
            .expect("Can't connect to data base");
        //Асинхронное подключение к бд при помощи tokio
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
                error!("Connection error: {}", e);
            }
        });
        client
    }

    //Реализация добавления заказа в базу данных, принимает структуру с заказом и пользователя, отправляющего запросы, возвращает Result
    pub async fn add_order(
        &self,
        input_data: Order,
        client: &Client,
    ) -> Result<(), Box<dyn Error>> {
        let add_delivery = format!(
            "insert into DELIVERY(NAME, PHONE, ZIP, CITY, ADDRESS, REGION, EMAIL)
            values ('{}', '{}', '{}', '{}', '{}', '{}', '{}')
            returning ID;",
            &input_data.delivery.name,
            &input_data.delivery.phone,
            &input_data.delivery.zip,
            &input_data.delivery.city,
            &input_data.delivery.address,
            &input_data.delivery.region,
            &input_data.delivery.email
        );
        debug!("Adding delivery sql request: {}", add_delivery);
        //Добавление Delivery в бд
        let delivery_rows = client.query(&add_delivery, &[]).await?;
        //Сохранение ид для добавления в таблицу с заказом
        let delivery_id: i32 = delivery_rows.get(0).unwrap().get("id");
        debug!("delivery_id: {}", delivery_id);
        let add_payment = format!("insert into PAYMENT(transaction, REQUEST_ID, CURRENCY, PROVIDER, AMOUNT, PAYMENT_DT, BANK, DELIVERY_COST, GOODS_TOTAL, CUSTOM_FEE)
            values ('{}', '{}', '{}', '{}', {}, {}, '{}', {}, {}, {})
            returning ID;", &input_data.payment.transaction, &input_data.payment.request_id, &input_data.payment.currency, &input_data.payment.provider, &input_data.payment.amount, &input_data.payment.payment_dt, &input_data.payment.bank, &input_data.payment.delivery_cost, &input_data.payment.goods_total, &input_data.payment.custom_fee);
        debug!("Adding payment sql request: {}", add_payment);
        //Добавление Payment в бд
        let payment_rows = client.query(&add_payment, &[]).await?;
        //Сохранение ид для добавления в таблицу с заказом
        let payment_id: i32 = payment_rows.get(0).unwrap().get("id");
        debug!("Payment_id: {}", payment_id);
        let add_order = format!("insert into ORDERS(ORDER_UID, TRACK_NUMBER, ENTRY, DELIVERY_ID, PAYMENT_ID, LOCALE, INTERNAL_SIGNATURE, CUSTOMER_ID, DELIVERY_SERVICE, SHARDKEY, SM_ID, DATE_CREATED, OOF_SHARD)
            values ('{}', '{}', '{}', {}, {}, '{}', '{}', '{}', '{}', '{}', {}, '{}', {})
            returning ID;", &input_data.order_uid, &input_data.track_number, &input_data.entry, &delivery_id, &payment_id, &input_data.locale, &input_data.internal_signature, &input_data.customer_id, &input_data.delivery_service, &input_data.shardkey, &input_data.sm_id, &input_data.date_created, &input_data.oof_shard);
        debug!("Adding order sql request: {}", add_payment);
        //Добавление Order в бд
        let order_rows = client.query(&add_order, &[]).await?;
        //Сохранение ид для добавления в таблицу с предметами, для сопоставления с заказом
        let order_id: i32 = order_rows.get(0).unwrap().get("id");
        debug!("Order_id: {}", order_id);
        for item in &input_data.items {
            //Добавление Item в бд
            let add_item = format!("insert into ITEMS(CHRT_ID, TRACK_NUMBER, PRICE, RID, NAME, SALE, size, TOTAL_PRICE, NM_ID, BRAND, STATUS, ORDER_ID)
                values ({}, '{}', {}, '{}', '{}', {}, '{}', {}, {}, '{}', {}, {});", &item.chrt_id, &item.track_number, &item.price, &item.rid, &item.name, &item.sale, &item.size, &item.total_price, &item.nm_id, &item.brand, &item.status, &order_id);
            debug!("Adding item sql request: {}", add_item);
            let item_rows = client.query(&add_item, &[]).await?;
        }
        Ok(())
    }

    //Получение данных из бд, принимает клиента для запросов в бд, возвращает Result
    pub async fn get_orders(&self, client: &Client) -> Result<Vec<Order>, Box<dyn Error>> {
        let mut ans: Vec<Order> = Vec::new();
        //Запрос для извлечения данных заказа, доставки и способа оплаты
        let rows = client
            .query(
                "select * from orders
            join payment on orders.payment_id = payment.id
            join delivery on orders.delivery_id = delivery.id;",
                &[],
            )
            .await?;
        for row in rows {
            let delivery = Delivery {
                name: row.get("name"),
                phone: row.get("phone"),
                zip: row.get("zip"),
                city: row.get("city"),
                address: row.get("address"),
                region: row.get("region"),
                email: row.get("email"),
            };
            let payment = Payment {
                transaction: row.get("transaction"),
                request_id: row.get("request_id"),
                currency: row.get("currency"),
                provider: row.get("provider"),
                amount: row.get("amount"),
                payment_dt: row.get("payment_dt"),
                bank: row.get("bank"),
                delivery_cost: row.get("delivery_cost"),
                goods_total: row.get("goods_total"),
                custom_fee: row.get("custom_fee"),
            };

            let mut items: Vec<Item> = Vec::new();
            let order_id: i32 = row.get("id");
            //Извлечение данных об элементах для каждого заказа
            let get_items = format!(
                "select * from ITEMS
            where ORDER_ID = {};",
                order_id
            );
            let item_rows = client.query(&get_items, &[]).await?;
            for item_row in item_rows {
                let item = Item {
                    chrt_id: item_row.get("chrt_id"),
                    track_number: item_row.get("track_number"),
                    price: item_row.get("price"),
                    rid: item_row.get("rid"),
                    name: item_row.get("name"),
                    sale: item_row.get("sale"),
                    size: item_row.get("size"),
                    total_price: item_row.get("total_price"),
                    nm_id: item_row.get("nm_id"),
                    brand: item_row.get("brand"),
                    status: item_row.get("status"),
                };
                items.push(item);
            }
            let order = Order {
                order_uid: row.get("order_uid"),
                track_number: row.get("track_number"),
                entry: row.get("entry"),
                delivery,
                payment,
                items,
                locale: row.get("locale"),
                internal_signature: row.get("internal_signature"),
                customer_id: row.get("customer_id"),
                delivery_service: row.get("delivery_service"),
                shardkey: row.get("shardkey"),
                sm_id: row.get("sm_id"),
                date_created: row.get("date_created"),
                oof_shard: row.get("oof_shard"),
            };
            debug!("Get new order");
            ans.push(order);
        }
        Ok(ans)
    }
}
