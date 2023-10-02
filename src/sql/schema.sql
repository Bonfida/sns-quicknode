CREATE TABLE IF NOT EXISTS provisioning (
    quicknode_id text,
    endpoint_id text,
    wss_url text,
    http_url text,
    referers text[],
    chain text,
    network text,
    plan text,
    expiry_timestamp bigint,
    CONSTRAINT provisioning_primary_key PRIMARY KEY(quicknode_id, endpoint_id)
);