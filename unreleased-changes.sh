#!/bin/sh

CRATE_TAGS=$(git tag | sed 's/-[^-]*$//' | uniq)
CARGOS=$(echo */*/Cargo.toml)

for TAG in $CRATE_TAGS; do
	LATEST_TAG=$(git tag | grep $TAG | sort -V | tail -n1)
	FOUND_CARGO=
	for CARGO in $CARGOS; do
		# fixups for tag names that don't match crate names
		CRATE=$(echo $TAG | sed -e 's/^jh$/jh-x86_64/' -e 's/^blake$/blake-hash/' -e 's/^groestl$/groestl-aesni/')
		if grep -q "name = \"$CRATE\"" $CARGO; then
			FOUND_CARGO=1
			DIR=$(dirname $CARGO)
			git log --color=always --stat $LATEST_TAG..HEAD -- $DIR | less -R
			break
		fi
	done
	if [ -z "$FOUND_CARGO" ]; then
		echo "Couldn't find a Cargo.toml for $TAG!"
	fi
done

