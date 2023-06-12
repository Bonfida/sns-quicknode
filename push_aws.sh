#!/bin/bash
aws lightsail \
push-container-image \
--region ap-southeast-1 \
--service-name qn-api \
--label main \
--image sns-quicknode:latest