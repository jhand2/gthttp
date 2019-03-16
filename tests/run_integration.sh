#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
TOOLS_DIR=$SCRIPT_DIR/../tools

docker-compose -f $TOOLS_DIR/docker-compose.yml up
docke-compose -f $TOOLS_DIR/docker-compose.yml down
