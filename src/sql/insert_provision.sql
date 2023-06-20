INSERT INTO provisioning VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
ON CONFLICT ON CONSTRAINT provisioning_primary_key 
DO UPDATE SET 
wss_url = EXCLUDED.wss_url,
http_url = EXCLUDED.http_url,
referers= EXCLUDED.referers,
chain = EXCLUDED.chain,
network = EXCLUDED.network,
plan = EXCLUDED.plan,
expiry_timestamp = EXCLUDED.expiry_timestamp;