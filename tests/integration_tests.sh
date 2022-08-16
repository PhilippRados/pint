#!/bin/bash

function assert_eq {
  if [ $# -eq 3 ]; then
    local snapshot=$1
    local fixture=$2
    # local cs=$3 # codelsize
    local name=$3
  else
    echo "assert_eq accepts 3 arguments"
    exit 1
  fi

  $(cargo r -q --release tests/fixtures/"$fixture" >& tmp)
  result=$(diff tests/snapshots/"$snapshot" tmp 2> err)
  error=$(cat err)

  if [[ "$result" = "" && "$error" = "" ]];
    then printf "\x1b[32mPASSED!\x1b[0m $name\n"
    else printf "\x1b[31mFAILED!\x1b[0m $name\nexpected: '$(cat tests/snapshots/"$snapshot")'\nactual: '$(cat tmp)'\n\n"
  fi
  rm tmp
  rm err
}

assert_eq "failure-input-file-not-found" "" "missing input"

# All truecolor-rgb and bit-depth 8
assert_eq "success_hello_world" "piet_hello_world.png" "piet_hello_world"
assert_eq "success_valentine" "valentines.png" "valentines"
assert_eq "success_fizzbuzz" "fizzbuzz.png" "fizzbuzz"

# indexed palette bit-depth 8
assert_eq "success_indexed_hello_world" "artsy_hello_world.png" "indexed_hello_world"
assert_eq "success_bottles" "99bottles.png" "99_bottles_indexed_switch"

# inferred codel-size
assert_eq "success_pi" "pi_big.png" "correct_codel_pi"
