# catscii

fasterthanlime series project with some personal modifications

Serves cat pictures as ascii art over the internet

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

