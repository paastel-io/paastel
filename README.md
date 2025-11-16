# PaaStel.io

1. docker-compose up -d
2. cargo binstall sqlx-cli -y && sqlx migrate run
3. cargo run

using insecure registry

```
cat /etc/docker/daemon.json
{
  "insecure-registries": ["0.0.0.0:5000"]
}
```
