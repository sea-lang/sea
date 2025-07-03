#!/usr/bin/env sh

cloc src std samples tests \
	--read-lang-def=res/cloc.txt \
	--fullpath \
	--not-match-d=std/hash/wyhash # exclude submodules
