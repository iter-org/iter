
## Setup

### Setup buildx
Follow guide: https://docs.docker.com/build/building/multi-platform/

## Building and pushing iter/ingress

Retrieve an authentication token and authenticate your Docker client to your registry. Use the AWS CLI:
```
aws ecr-public get-login-password --region us-east-1 | docker login --username AWS --password-stdin public.ecr.aws/k2s9w9h5
```

Build your Docker image using the following command. For information on building a Docker file from scratch see the instructions here. You can skip this step if your image is already built:
```
docker buildx build --platform linux/amd64 . -t iter/ingress --load
docker buildx build --platform linux/arm64 . -t iter/ingress --load
```

After the build completes, tag your image so you can push the image to this repository:
```
docker tag iter/ingress:latest public.ecr.aws/k2s9w9h5/iter/ingress:latest
```

Run the following command to push this image to your newly created AWS repository:
```
docker push public.ecr.aws/k2s9w9h5/iter/ingress:latest
```