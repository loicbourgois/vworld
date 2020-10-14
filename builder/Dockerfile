FROM docker:latest

COPY vworld-server/ .
COPY scripts/vworld-build.sh .
RUN chmod +x vworld-build.sh

CMD ["vworld-build.sh"]
