UPDATE provisioning SET 
    expiry_timestamp = $1
WHERE quicknode_id = $2 AND endpoint_id = $3