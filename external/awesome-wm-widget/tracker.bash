#!/bin/bash

set -eo pipefail

tracker=$(bash -c 'curl -s localhost:8000/tracker')
key=$(jq -r .key <<< "${tracker}")
description=$(jq -r --arg key "${key}" '.[] | select(.key == $key) | .fields.summary' ~/.jira/issues.json)
jq -r --arg description "${description}" '"[" + .key + "] " + $description + ": " + .duration' <<< "${tracker}"
