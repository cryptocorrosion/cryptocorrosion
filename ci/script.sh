# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release
}

portable_only() {
    cross build --target $TARGET -p c2-chacha -p ppv-lite86
    cross build --target $TARGET --release -p c2-chacha -p ppv-lite86

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET -p c2-chacha -p ppv-lite86
    cross test --target $TARGET --release -p c2-chacha -p ppv-lite86
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    if [ -z $PORTABLE_ONLY ]; then
        main
    else
        portable_only
    fi
fi
