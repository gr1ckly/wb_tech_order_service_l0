create table PAYMENT(
                        id serial primary key,
                        transaction text not null,
                        request_id text not null,
                        currency text not null,
                        provider text not null,
                        amount int not null,
                        payment_dt bigint not null,
                        bank text not null,
                        delivery_cost int not null,
                        goods_total int not null,
                        custom_fee int not null
);

create table DELIVERY(
                         ID SERIAL primary key,
                         NAME text not null,
                         PHONE text not null,
                         ZIP text not null,
                         CITY text not null,
                         ADDRESS text not null,
                         REGION text not null,
                         EMAIL text not NULL
);

create table ORDERS(
                       ID SERIAL primary key,
                       ORDER_UID text not null,
                       TRACK_NUMBER text not null,
                       ENTRY text not null,
                       DELIVERY_ID INT not null references DELIVERY (ID),
                       PAYMENT_ID INT not null references PAYMENT (ID),
                       LOCALE text not null,
                       INTERNAL_SIGNATURE text not null,
                       CUSTOMER_ID text not null,
                       DELIVERY_SERVICE text not null,
                       SHARDKEY text not null,
                       SM_ID INT not null,
                       DATE_CREATED text not null,
                       OOF_SHARD text not NULL
);

create table ITEMS(
                      ID SERIAL primary key,
                      CHRT_ID BIGINT not null,
                      TRACK_NUMBER text not null,
                      PRICE INT not null,
                      RID text not null,
                      NAME text not null,
                      SALE INT not null,
                      size text not null,
                      TOTAL_PRICE INT not null,
                      NM_ID BIGINT not null,
                      BRAND text not null,
                      STATUS INT not null,
                      ORDER_ID INT not null REFERENCES ORDERS (ID)
);