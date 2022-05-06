-- Add migration script here
CREATE TABLE agreement_details(
    role_id  char(1) not null,
    node_id varchar(42) not null,
    agreement_id varchar(120) not null,
    peer_id varchar(42) not null,
    created_ts TIMESTAMPTZ not null,
    valid_to TIMESTAMPTZ,
    runtime varchar(50),
    payment_platform varchar(50),
    payment_address varchar(50),
    subnet varchar(120),
    task_package varchar(300),
    CONSTRAINT agreement_details_pk PRIMARY KEY (role_id, node_id, agreement_id)
);


