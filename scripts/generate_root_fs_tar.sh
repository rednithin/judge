#!/bin/bash

set -e

docker build -f RootFS.Dockerfile -t judge_root_fs .
CONTAINER_RUN_ID=$(docker run -d judge_root_fs:latest)
echo "CONTAINER_RUN_ID: $CONTAINER_RUN_ID"
docker export "$CONTAINER_RUN_ID" > tar/judge_root_fs.tar
echo "Tar file successfully generated"