#!/bin/zsh
for f in *epe; do ./convert.sh $f > ${f//.epe/.js}; done
