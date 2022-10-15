#!/bin/sh
jq -r < $1 .sources.main
