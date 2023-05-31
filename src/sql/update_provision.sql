UPDATE provisioning SET 
    endpoint_id = $1,
    wss_url = $2,
    http_url = $3,
    referers = $4,
    chain = $5,
    network = $6,
    plan = $7,
    expiry_timestamp = $8
WHERE quicknode_id = $9