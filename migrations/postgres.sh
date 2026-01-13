docker pull postgres:17.7

# 创建用于存储 PostgreSQL 数据的目录
sudo mkdir -p /var/lib/postgresql/data
# 设置适当的权限
sudo chown -R 999:999 /var/lib/postgresql/data

docker run -d \
  --name postgres \
  --restart unless-stopped \
  -e TZ=Asia/Shanghai \
  -e POSTGRES_PASSWORD=xiaolu \
  -e PGDATA=/var/lib/postgresql/data/pgdata \
  -p 5432:5432 \
  -v /var/lib/postgresql/data:/var/lib/postgresql/data \
  postgres:17.7 \
  -c shared_buffers=256MB \
  -c max_connections=200
