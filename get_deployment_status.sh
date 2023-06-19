#!/bin/bash
aws lightsail \
get-container-service-deployments \
--region ap-southeast-1 \
--service-name qn-api \
--query 'deployments[:3].{State: state, Version: version, Image: containers."sns-quicknode".image}' | \
jq --raw-output '(["STATE","VERSION", "IMAGE"] | (., map(length*"-"))), (.[] | [.State, .Version, .Image]) | @tsv' | column -ts $'\t'
# --query "deployments[0]" | cat
# --label main \
# --image sns-quicknode:latest