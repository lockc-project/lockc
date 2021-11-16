# Container image

lockc repository contains a `Dockerfile` which can be used for building a
container image. The main purpose of building it is ability to deploy lockc on
Kubernetes.

Building a local image can be done in a basic way, like:

```bash
docker build -t lockcd .
```

For quick development and usage of the image on different (virtual) machines,
it's convenient to use [ttl.sh](https://ttl.sh/) which is an anonymous and
ephemeral container image registry.

To build and push an image to ttl.sh, you can use the following commands:

```bash
export IMAGE_NAME=$(uuidgen)
docker build -t ttl.sh/${IMAGE_NAME}:30m .
docker push ttl.sh/${IMAGE_NAME}:30m
```

After building the container image, you will be able to
[install lockc on Kubernetes](../install/kubernetes.md).
