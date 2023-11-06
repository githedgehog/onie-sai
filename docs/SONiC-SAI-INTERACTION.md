# SONiC and SAI Interaction

Here is a list of things on how you can get more information from within SONiC about what the actual SAI state is on the switch.
SONiC interacts with SAI through a specific redis database, and internally the SONiC syncd process is what will actually communicate with the SAI library to program the switch.
The confusing part here is that there is a "SAI redis client" whose API is _exactly_ like SAI, but it will actually only write things into redis for it to be synced to the chip by syncd.

So here are some sections on how to interact with everything.

## Dumping the ASIC_STATE

SONiC has a tool called `redis-dump` which are wrappers for the right docker container to execute into it and dumping some of the redis databases.
The difficult part seems to be (and this might be Broadcom SONiC specific) is _how_ to dump the ASIC state as the official instructions from the documentation are wrong and do not work.

First, you need to gather the password of the database.
Shell into the running database container:

```shell
docker exec -ti database /bin/bash
```

And get the password by extracting it from the redis database:

```shell
cat /etc/redis/redis.conf | grep "requirepass"
```

You will see a base64-encoded string which is the cleartext password of the database.

Next thing is that you need to know on which redis database server and database number the ASIC_STATE is located.
In our case we found it to be on database 1 on the redis database server which can be accessed through port 63793.

This means that as the admin user from a SONiC shell prompt, you can dump the whole ASIC_STATE like the following, replacing PASSWORD with the password that you extracted from the redis configuration file above:

```shell
redis-dump -p 63793 -d 1 -w "PASSWORD" -k "*ASIC_STATE*" -y > asic_state.json
```

If you just want to query specific objects and/or attributes, you can also simply connect to the redis-clie and issue commands like the following:

```shell
redis-cli -p 63793 -n 1
HGETALL ASIC_STATE:SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000003523
HGET ASIC_STATE:SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000003523 SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE
```

## Monitor ASIC Updates

Another way to monitor what exactly is being updated in the chip is by simply using the redist `MONITOR` command to watch for all the redis interactions from which one can extract what changes are being made to the ASIC.

To understand what one is seeing when using the redis monitor command, one needs to understand how the "SAI redis client" and "SAI redis server" implementations work together:
SONiC is using a combination of redis pub/sub and a separate list entry for queueing updates to push updates to the chip.
The client will push an operation entry to the `ASIC_STATE_KEY_VALUE_OP_QUEUE` list entry.
This could look like the following:

```redis
"LPUSH" "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000a23" "[\"SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE\",\"true\"]" "Sset"
```

It can push multiple of these operations all with statements like the one above.
To signal the server that there are updates which need to be processed, it will then use pub-sub to notify the server of operations that need to be processed.
You will see statements like the following:

```redis
"PUBLISH" "ASIC_STATE_CHANNEL" "G"
```

Note that it is in fact a bit more complicated: the operations are queued through on-the-fly generated in-memory Lua scripts which are being executed within redis.
So you will see an operation like the following:

```redis
"EVALSHA" "d171e04fd79e95ca2287f3b067c46ae76a82208b" "2" "ASIC_STATE_KEY_VALUE_OP_QUEUE" "ASIC_STATE_CHANNEL" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000a23" "[\"SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE\",\"true\"]" "Sset" "G"
```

Once syncd has processed the commands and used the _actual_ SAI to run the command, it will update the appropriate `ASIC_STATE` entry in redis to reflect the change.
For example, to enable IPv4 multicast on a router interface the `SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE` attribute of a router interface must be set to `true`.
You will see all the following commands for that when monitoring redis:

```redis
"EVALSHA" "d171e04fd79e95ca2287f3b067c46ae76a82208b" "2" "ASIC_STATE_KEY_VALUE_OP_QUEUE" "ASIC_STATE_CHANNEL" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000a23" "[\"SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE\",\"true\"]" "Sset" "G"
 "LPUSH" "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000a23" "[\"SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE\",\"true\"]" "Sset"
 "PUBLISH" "ASIC_STATE_CHANNEL" "G"
"EVALSHA" "22b6eee2f212572e878369f340e8f7079c77cf7e" "2" "ASIC_STATE_KEY_VALUE_OP_QUEUE" "ASIC_STATE" "128" "1"
 "LRANGE" "ASIC_STATE_KEY_VALUE_OP_QUEUE" "-384" "-1"
 "LTRIM" "ASIC_STATE_KEY_VALUE_OP_QUEUE" "0" "-385"
 "HSET" "ASIC_STATE:SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000a23" "SAI_ROUTER_INTERFACE_ATTR_V4_MCAST_ENABLE" "true"
```

Keeping this information in mind, you can follow all the SAI operations which are being executed by simply activating the redis monitor with the following commands:

```shell
redis-cli -p 63793 -n 1
MONITOR
```

## Making SAI changes in SONiC through redis-cli

If you want to make changes to SAI directly in SONiC, you can do this therefore by acting like a "SAI redis client".
You can do this by pushing all the change operations that you want to make to the `ASIC_STATE_KEY_VALUE_OP_QUEUE` list entry, and then triggering the processing through the pub-sub notification mechanism.

For example, if you want to enable IPv6 multicast on a set of router interfaces, you can connect to the redis-cli, and then run the following commands:

```shell
redis-cli -p 63793 -n 1
LPUSH "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x60000000009f9" "[\"SAI_ROUTER_INTERFACE_ATTR_V6_MCAST_ENABLE\",\"true\"]" "Sset"
LPUSH "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x60000000009f8" "[\"SAI_ROUTER_INTERFACE_ATTR_V6_MCAST_ENABLE\",\"true\"]" "Sset"
LPUSH "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000a23" "[\"SAI_ROUTER_INTERFACE_ATTR_V6_MCAST_ENABLE\",\"true\"]" "Sset"
LPUSH "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000952" "[\"SAI_ROUTER_INTERFACE_ATTR_V6_MCAST_ENABLE\",\"true\"]" "Sset"
LPUSH "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x6000000000a26" "[\"SAI_ROUTER_INTERFACE_ATTR_V6_MCAST_ENABLE\",\"true\"]" "Sset"
LPUSH "ASIC_STATE_KEY_VALUE_OP_QUEUE" "SAI_OBJECT_TYPE_ROUTER_INTERFACE:oid:0x60000000009f7" "[\"SAI_ROUTER_INTERFACE_ATTR_V6_MCAST_ENABLE\",\"true\"]" "Sset"
PUBLISH "ASIC_STATE_CHANNEL" "G"
```

## More Information

To understand how this is all working in SONiC the best address is to look at everything "consumer" and "producer" related in the `sonic-swss-common` repository.
For example here: [https://github.com/sonic-net/sonic-swss-common/blob/master/common/consumerstatetable.cpp]
