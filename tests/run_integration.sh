#!/bin/bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
LOG_DIR=$SCRIPT_DIR/logs

COMPOSE="docker-compose --no-ansi -f $SCRIPT_DIR/docker-compose.yml"

mkdir -p $LOG_DIR

lookup_docker_ip() {
    IP=$($COMPOSE exec -T gthttp ping -c 1 $1 | head -n1 | grep -o -E '(([0-9]+\.){3}[0-9]+)')
    echo "$IP"
}

run_tests() {
    echo "Running tests"

    IP1=$(lookup_docker_ip "target1")
    IP2=$(lookup_docker_ip "target2")

    # Target static IPs of the alpine machines
    $COMPOSE exec gthttp gthttp eth0 $IP1 $IP2
}

up() {
    echo "Bringing up test infrastructure"
    $COMPOSE up &> $LOG_DIR/test_setup.txt &
}

down() {
    echo "Tearing down test infrastructure"
    $COMPOSE down &> $LOG_DIR/test_teardown.txt
}

CMD=$1

case $CMD in
    up)
        up
        ;;
    down)
        down
        ;;
    test)
        run_tests
        ;;
    full)
        up
        run_tests
        down
        ;;
    *)
        echo "Invalid command $CMD"
esac

