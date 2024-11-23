# catscii

A modified [fasterthanlime](https://fasterthanli.me/) series project

Serves cat pictures as ascii art over the internet. Also geolocates by country, but this only works when deployed due to dependency on a fly.io specific header.

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

