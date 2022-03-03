-- Add migration script here
CREATE TABLE AGREEMENT_STATUS(
  role_id  char(1) not null,
  node_id varchar(42) not null,
  agreement_id varchar(120) not null,
  requested decimal not null default 0,
  accepted decimal not null default 0,
  confirmed decimal not null default 0,
  created_ts  timestamp without time zone not null DEFAULT CURRENT_TIMESTAMP,
  updated_ts  timestamp without time zone not null DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT AS_PK PRIMARY KEY(role_id, node_id, agreement_id),
  CONSTRAINT AS_ROLE_CHK CHECK(role_id in ('R', 'P'))
);


