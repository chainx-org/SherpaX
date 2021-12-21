#!/bin/bash
VERSION=$(./target/release/sherpax --version | awk '{print $2}' | awk -F'-' '{print $1}')

print_txt () {
	echo  "md5sum:"
	md5sum ./target/release/sherpax | echo "`awk '{print $1}'` sherpax-$VERSION-ubuntu-20.04-x86_64"
	md5sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.wasm | echo "`awk '{print $1}'` sherpax_wasm"
	md5sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.wasm | echo "`awk '{print $1}'` sherpax_compact_wasm"
	md5sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.compressed.wasm | echo "`awk '{print $1}'` sherpax_compressed_wasm"

	echo  "sha256:"
	sha256sum ./target/release/sherpax | echo "`awk '{print $1}'` sherpax-$VERSION-ubuntu-20.04-x86_64"
	sha256sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.wasm | echo "`awk '{print $1}'` sherpax_wasm"
	sha256sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.wasm | echo "`awk '{print $1}'` sherpax_compact_wasm"
	sha256sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.compressed.wasm | echo "`awk '{print $1}'` sherpax_compressed_wasm"
}

print_markdown () {
	md5sum_sherpax=$(md5sum ./target/release/sherpax | awk '{print $1}')
	md5sum_wasm=$(md5sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.wasm | awk '{print $1}')
	md5sum_compact=$(md5sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.wasm | awk '{print $1}')
	md5sum_compressed=$(md5sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.compressed.wasm | awk '{print $1}')

	sha256sum_sherpax=$(sha256sum ./target/release/sherpax | awk '{print $1}')
	sha256sum_wasm=$(sha256sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.wasm | awk '{print $1}')
	sha256sum_compact=$(sha256sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.wasm | awk '{print $1}')
	sha256sum_compressed=$(sha256sum ./target/release/wbuild/sherpax-runtime/sherpax_runtime.compact.compressed.wasm | awk '{print $1}')

	echo "| md5sum | sha256 | name |
| :---: | :-----: | :-----: |
|$md5sum_sherpax|$sha256sum_sherpax|sherpax-$VERSION-ubuntu-20.04-x86_64|
|$md5sum_wasm|$sha256sum_wasm|sherpax_wasm|
|$md5sum_compact|$sha256sum_compact|sherpax_compact_wasm|
|$md5sum_compressed|$sha256sum_compressed|sherpax_compressed_wasm|
"
}


if [ "$1" = md ]; then
    print_markdown
else
    print_txt
fi
