-- Add migration script here
ALTER TABLE AGREEMENT_STATUS ADD PEER_ID varchar(42);

ALTER TABLE AGREEMENT_STATUS ADD REPORTED_TS TIMESTAMPTZ;