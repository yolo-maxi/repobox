# repo.box publisher probe: a dependency-free Node app used by the publisher
# end-to-end tests. Build and upload it like any published app:
#   docker build --platform linux/amd64 -t publisher-probe . && docker save publisher-probe | gzip > probe.tar.gz
FROM node:22-alpine
WORKDIR /app
COPY server.js index.html ./
# /data is a platform volume; Docker initialises a new volume from this
# directory, so the non-root runtime user can write it.
RUN mkdir -p /data && chown node:node /data
USER node
EXPOSE 8080
CMD ["node", "server.js"]
