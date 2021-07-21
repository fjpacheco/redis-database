#!/bin/bash
# Ejecutar server con "make run_v" para visualizar el server con valgrind.
# Luego el script ejecutarlo con "./run_garbage.sh" en TODAS las terminales que quieras 😁😁😁

# Para testearer que ande todo bien, ejecutar minimo 5~10 termianles con este script 
# ..y la 6° terminal conectarse con "nc localhost 6379" y ver que tal va todo, hacer un shutdown por ejemplo...

# Ojo con la persistencia y este contador! 
# Descomentarlo para probar.... se irá agrandando demasiado la database y la RAM hará panic con rayos cósmicos ☢ 
counter = 0
while echo "set key value"; do
  ((counter=counter+1))
  echo "set" $counter "1"
  #echo "expire" $counter 10
  echo "get" $counter
 done > >(nc localhost 6379)