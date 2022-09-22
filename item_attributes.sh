#!/usr/bin/env bash

TMP=$(mktemp)

for F in $(find assets/data/json/items/ -type f) ; do
    for LINE in $(jq -r '.[] | [(keys | join(" ") | sub("~"; "(tilde)")), .type] | @csv' $F | sed -e 's#"##g' -e 's# #~#g') ; do
        #echo -e "$LINE"
        ATTRIBUTES=${LINE%%,*}
        TYPE=${LINE##*,}
        echo "$ATTRIBUTES" | sed -E -e "s#(~|\$)#~$TYPE\n#g" >> $TMP
    done
done

LC_COLLATE=C
for LINE in $(cat $TMP | grep -v "^$" | sort -u | sed -e 's#~#,#g' -e 's#(tilde)#~#g') ; do
    #echo -e "$LINE"
    ATTRIBUTE=${LINE%%,*}
    TYPE=${LINE##*,}
    if grep --quiet "$ATTRIBUTE" src/cdda/data/cdda_item_info.rs ; then
        #echo "$ATTRIBUTE exists"
        :
    else
        echo "$ATTRIBUTE not found - $TYPE"
    fi
done

rm $TMP
