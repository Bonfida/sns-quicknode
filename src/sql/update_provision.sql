UPDATE provisioning SET 
    wss_url = $2,
    http_url = $3,
    referers = $4,
    chain = $5,
    network = $6,
    plan = $7,
    expiry_timestamp = $8
WHERE quicknode_id = $9 AND endpoint_id = $1