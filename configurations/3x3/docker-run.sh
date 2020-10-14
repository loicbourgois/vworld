# vworld-server-1
      docker rm -f vworld-server-1;
      docker run \
        --detach \
        --env port=10001 \
        --publish 10001:10001 \
        --name "vworld-server-1" \
        "vworld-server";
    # vworld-server-2
      docker rm -f vworld-server-2;
      docker run \
        --detach \
        --env port=10002 \
        --publish 10002:10002 \
        --name "vworld-server-2" \
        "vworld-server";
    # vworld-server-3
      docker rm -f vworld-server-3;
      docker run \
        --detach \
        --env port=10003 \
        --publish 10003:10003 \
        --name "vworld-server-3" \
        "vworld-server";
    # vworld-server-4
      docker rm -f vworld-server-4;
      docker run \
        --detach \
        --env port=10004 \
        --publish 10004:10004 \
        --name "vworld-server-4" \
        "vworld-server";
    # vworld-server-5
      docker rm -f vworld-server-5;
      docker run \
        --detach \
        --env port=10005 \
        --publish 10005:10005 \
        --name "vworld-server-5" \
        "vworld-server";
    # vworld-server-6
      docker rm -f vworld-server-6;
      docker run \
        --detach \
        --env port=10006 \
        --publish 10006:10006 \
        --name "vworld-server-6" \
        "vworld-server";
    # vworld-server-7
      docker rm -f vworld-server-7;
      docker run \
        --detach \
        --env port=10007 \
        --publish 10007:10007 \
        --name "vworld-server-7" \
        "vworld-server";
    # vworld-server-8
      docker rm -f vworld-server-8;
      docker run \
        --detach \
        --env port=10008 \
        --publish 10008:10008 \
        --name "vworld-server-8" \
        "vworld-server";
    # vworld-server-9
      docker rm -f vworld-server-9;
      docker run \
        --detach \
        --env port=10009 \
        --publish 10009:10009 \
        --name "vworld-server-9" \
        "vworld-server";
    