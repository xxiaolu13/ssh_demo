#!/usr/bin/fish
sudo mkdir -p /var/lib/redis/data
sudo chown -R 999:999 /var/lib/redis/data
docker pull redis:7.2
docker run -d \
  --name redis-dev \
  --restart always \
  -p 6379:6379 \
  -v /var/lib/redis/data:/data \
  redis:7.2 \
  redis-server --appendonly yes --requirepass "xiaolu"
