#!/bin/bash

# A simple script to add garbage in the database.
counter = 0
while echo "set key value"; do
  ((counter=counter+1))
  echo "set" $counter "1"
  echo "lpush" "lista" $counter
  echo "sadd" "set" $counter
  echo "get" $counter
 done > >(nc localhost 6379)