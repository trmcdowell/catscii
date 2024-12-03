# catscii

My version of catscii, an application from a [fasterthanlime series](https://fasterthanli.me/series/building-a-rust-service-with-nix).

Serves cat pictures as ascii art over the internet. Also geolocates by country, but this only works when deployed due to dependency on a fly.io specific header.

If you want to build this application, please note that you need to create your own `.env` and `fly.toml` files because the ones included in this repository are encrypted with [git-crypt](https://github.com/AGWA/git-crypt).

Build Docker image
```
docker build --tag catscii .
```

Run Docker image locally
```
docker run --env-file ./.env -it -p <port>:<port>/tcp --rm catscii 
```

Deploy to [fly.io](https://fly.io/)
```
fly deploy --local-only
```

