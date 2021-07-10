#!/bin/bash

echo "Hello"
function func() {
  echo "Task completed"
}
trap func XCPU
ulimit -St 5


strace -e 'trace=!all' /usr/bin/python /home/nithin/Git/judge/abc.py