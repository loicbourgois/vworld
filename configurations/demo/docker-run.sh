# vworld-server-1
      docker rm -f vworld-server-1;
      docker run \
         \
        --tty \
        --env address=0.0.0.0         --env port=10001 \
        --env configuration='{"particles":[{"type":"sun","x":0.5}],"x":0,"y":0}' \
        --publish 10001:10001 \
        --name "vworld-server-1" \
        "vworld-server";
    