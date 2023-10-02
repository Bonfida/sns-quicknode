#!/bin/bash
aws lightsail \
get-container-service-deployments \
--region ap-southeast-1 \
--service-name qn-api \
--query "deployments[0]"
# --label main \
# --image sns-quicknode:latest