#!/bin/bash
IMAGE_ID=$(aws lightsail \
push-container-image \
--region ap-southeast-1 \
--service-name qn-api \
--label main \
--image sns-quicknode:latest \
| grep 'Refer to this image as ".*"' | awk '{gsub("\"", ""); print $6}')
echo $IMAGE_ID
aws lightsail \
create-container-service-deployment \
--cli-input-json "$(cat new_deployment_skeleton.json | awk -v image_id=$IMAGE_ID '{gsub ("IMAGE_ID", image_id); print}')" \
--region ap-southeast-1