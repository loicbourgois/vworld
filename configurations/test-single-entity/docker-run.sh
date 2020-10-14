# vworld-server-1
      docker rm -f vworld-server-1;
      docker run \
         \
        --tty \
        --env address=0.0.0.0         --env port=10001 \
        --env chunk_configuration='{"entities":[{"particles":[{"x":0.5,"y":0.5,"diameter":0.02,"mass":0.02,"type_":"default"},{"x":0.52,"y":0.5,"diameter":0.02,"mass":0.02,"type_":"muscle"},{"x":0.54,"y":0.5,"diameter":0.02,"mass":0.02,"type_":"muscle"},{"x":0.56,"y":0.5,"diameter":0.02,"mass":0.02,"type_":"default"}],"links":[{"pids":[0,1],"strengh":0},{"pids":[1,2],"strengh":0},{"pids":[2,3],"strengh":0}]}],"x":0,"y":0}' \
        --publish 10001:10001 \
        --name "vworld-server-1" \
        "vworld-server";
    