# Development Workflow

```shell
telnet 192.168.88.10 7016
scp said ubuntu@192.168.88.205:/var/www/html/marcus/
ssh root@192.168.88.158
cd saictl/bin/
rm said && wget http://192.168.88.205/marcus/said && chmod +x said
LD_LIBRARY_PATH=/root/saictl/lib ./said
```
