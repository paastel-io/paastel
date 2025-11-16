# PaaStel.io

1. docker-compose up -d
2. cargo binstall sqlx-cli -y && sqlx migrate run
3. cargo run


```
git remote add paastel ssh://git@localhost:2222/kovi/devsecops/site-estatico.git
git add -A && git branch -M main && git commit -m "Init" && git push paastel main
```

using insecure registry

```
cat /etc/docker/daemon.json
{
  "insecure-registries": ["0.0.0.0:5000"]
}
```
