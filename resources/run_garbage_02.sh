#!/bin/bash

# A simple script to add garbage to the database that expires after 10 seconds.
counter = 0
while echo "set key value"; do
  ((counter=counter+1))
  echo "set" $counter "1"
  echo "lpush" "lista" $counter
  echo "sadd" "set" $counter
  echo "expire" $counter 10
  echo "get" $counter
 done > >(nc localhost 6379)