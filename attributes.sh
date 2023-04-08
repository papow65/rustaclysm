#!/usr/bin/env bash

for F in $(find assets/data/json/ -type f) ; do
    for LINE in $(jq -r '.[] | [(keys | join(" ") | sub("~"; "(tilde)")), .type] | @csv' $F | sed -e 's#"##g' -e 's# #~#g') ; do
        #echo -e "$LINE"
        ATTRIBUTES=${LINE%%,*}
        TYPE=${LINE##*,}
        echo "$ATTRIBUTES@$TYPE" | sed -E -e "s#~#@$TYPE\n#g"
    done
done
