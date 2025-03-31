# Konobangu Testing Torrents Container

## Build

```bash
docker buildx build --platform linux/amd64 --tag konobangu-testing-torrents:latest --load .
```

## Run

```bash
docker run --network_mode=host --name konobangu-testing-torrents konobangu-testing-torrents:latest
```

## Publish

```bash
docker tag konobangu-testing-torrents:latest ghcr.io/dumtruck/konobangu-testing-torrents:latest
docker push ghcr.io/dumtruck/konobangu-testing-torrents:latest
```