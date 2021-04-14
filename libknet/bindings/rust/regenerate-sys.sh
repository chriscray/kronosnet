#
# Regerate the FFI bindings in src/sys from the current Corosync headers
#
regen()
{
    bindgen \
	--size_t-is-usize \
	--no-recursive-whitelist \
	--no-prepend-enum-name \
	--no-layout-tests \
	--no-doc-comments \
	--generate functions,types,vars \
	--fit-macro-constant-types \
	--whitelist-var=$2.*  \
	--whitelist-type=.* \
	--whitelist-function=*. \
	../../$1.h -o src/sys/$1.rs
}


regen libknet KNET
