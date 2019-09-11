cmake_minimum_required(VERSION 3.0)
project (csocks C)

add_executable (checkopts checkopts.c)

install (TARGETS checkopts DESTINATION .)
