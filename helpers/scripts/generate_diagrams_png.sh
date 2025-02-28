#!/bin/sh

dir=docs

for file in $dir/*.d2
do
    file_name=$(basename $file .d2)
    echo "Generating diagram for $file_name"
    d2 $dir/$file_name.d2 $dir/$file_name.svg
    inkscape $dir/$file_name.svg -o $dir/$file_name.png > /dev/null

done
