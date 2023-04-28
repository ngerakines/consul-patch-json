#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

if ! command -v curl &> /dev/null
then
    echo "ERROR curl could not be found"
    exit
fi

if ! command -v jq &> /dev/null
then
    echo "ERROR jq could not be found"
    exit
fi


if ! command -v jo &> /dev/null
then
    echo "ERROR jo could not be found"
    exit
fi

if ! command -v consul &> /dev/null
then
    echo "ERROR consul could not be found"
    exit
fi

CONSUL="http://localhost:8500"


curl -s --output /dev/null $CONSUL/v1/agent/host || { echo "ERROR Consul is unreachable" ; exit 1; }

# --- replace attribute

consul kv put integration/first '{"message":"hello world"}' > /dev/null
consul kv get integration/first | grep -q '{"message":"hello world"}' || { echo "ERROR Consul key integration/first incorrect" ; exit 1; }

$1 integration/first --dry-run message='"hello nick"' | jq '.message' | grep -q '"hello nick"' && echo "OK integration/first dry-run" || { echo "ERROR Consul key integration/first dry-run incorrect" ; exit 1; }
$1 integration/first message='"hello nick"' && echo "OK integration/first updated" || { echo "ERROR Consul key integration/first failed" ; exit 1; }

consul kv get integration/first | grep -q '{"message":"hello nick"}' && echo "OK integration/first verified" || { echo "ERROR Consul key integration/first not updated" ; exit 1; }

# --- add attribute

consul kv put integration/second '{"message":"hello world"}' > /dev/null
DATE="$(date "+%D")"

# jo message="hello world" | base64
$1 integration/second --dry-run date="\"${DATE}\"" | jq '.date' | grep -q "$DATE" && echo "OK integration/second dry-run" || { echo "ERROR Consul key integration/second dry-run incorrect" ; exit 1; }
$1 integration/second date="\"${DATE}\"" && echo "OK integration/second updated" || { echo "ERROR Consul key integration/second failed" ; exit 1; }

consul kv get integration/second | grep -q "{\"date\":\"${DATE}\",\"message\":\"hello world\"}" && echo "OK integration/second verified" || { echo "ERROR Consul key integration/second not updated" ; exit 1; }

# --- add complex

consul kv put integration/third '{"message":"hello world"}' > /dev/null

[[ $($1 integration/third --dry-run coordinates='[{"x": 39.759444, "y": -84.191667}]' | jq '.' -c) == '{"coordinates":[{"x":39.759444,"y":-84.191667}],"message":"hello world"}' ]] && echo "OK integration/third dry-run" || { echo "ERROR Consul key integration/third dry-run incorrect" ; exit 1; }

$1 integration/third coordinates='[{"x": 39.759444, "y": -84.191667}]' && echo "OK integration/third updated" || { echo "ERROR Consul key integration/third failed" ; exit 1; }

[[ $(consul kv get integration/third | jq '.' -c) == '{"coordinates":[{"x":39.759444,"y":-84.191667}],"message":"hello world"}' ]] && echo "OK integration/third verified" || { echo "ERROR Consul key integration/third not updated" ; exit 1; }

# -- add stdin

consul kv put integration/fourth '{"message":"hello world"}' > /dev/null

[[ $(echo '"nice"' | $1 integration/fourth --dry-run status=-- | jq '.' -c) == '{"message":"hello world","status":"nice"}' ]] && echo "OK integration/fourth dry-run" || { echo "ERROR Consul key integration/fourth dry-run incorrect" ; exit 1; }

echo '"nice"'| $1 integration/fourth status=-- && echo "OK integration/fourth updated" || { echo "ERROR Consul key integration/fourth failed" ; exit 1; }

[[ $(consul kv get integration/fourth | jq '.' -c) == '{"message":"hello world","status":"nice"}' ]] && echo "OK integration/fourth verified" || { echo "ERROR Consul key integration/fourth not updated" ; exit 1; }

# -- merge stdin

consul kv put integration/fifth '{"message":"hello world"}' > /dev/null

[[ $(jo coffee=good | $1 integration/fifth --dry-run -- | jq '.' -c) == '{"coffee":"good","message":"hello world"}' ]] && echo "OK integration/fifth dry-run" || { echo "ERROR Consul key integration/fifth dry-run incorrect" ; exit 1; }

jo coffee=good | $1 integration/fifth -- && echo "OK integration/fourth updated" || { echo "ERROR Consul key integration/fifth failed" ; exit 1; }

[[ $(consul kv get integration/fifth | jq '.' -c) == '{"coffee":"good","message":"hello world"}' ]] && echo "OK integration/fifth verified" || { echo "ERROR Consul key integration/fifth not updated" ; exit 1; }

# consul kv get integration/first
# consul kv get integration/second
# consul kv get integration/third
# consul kv get integration/fourth
# consul kv get integration/fifth
