#!/usr/bin/env bash

for F in $(find assets/data/json/ -type f) ; do
    for LINE in $(jq -r '.[] | [(keys | join(" ") | sub("~"; "(tilde)")), .type, .subtype] | @csv' $F | sed -e 's#"##g' -e 's# #~#g') ; do
        #echo -e "$LINE"
        readarray -d, -t arr <<<"$LINE,"
        ATTRIBUTES="${arr[0]}"
        TYPE="${arr[1]}"
        SUBTYPE="${arr[2]}"
        #echo "$ATTRIBUTES@${TYPE}@${SUBTYPE}"
        echo "$ATTRIBUTES@$TYPE@$SUBTYPE" | sed -E -e "s#~#@${TYPE}@${SUBTYPE}\n#g"
    done
done
