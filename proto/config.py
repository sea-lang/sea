import pathlib

# ANCHOR: Versioning

VERSION = '0.0.1-dev'

# Version naming scheme for the current, past, and future versions.
# If we go to 1.0 before 0.5, then all names above 0.5 will be shifted.
VERSION_SCHEME = {
	'0.0': 'prototype',
	'0.1': 'humuhumunukunukuapua\'a', # This is my favourite fish :>
	'0.2': 'lobster',
	'0.3': 'jellyfish',
	'0.4': 'seahorse',
	'0.5': 'dolphin',
	'1.0': 'coral',
}

VERSION_KIND = 'dev' if '-dev' in VERSION else ('beta' if VERSION[0] == '0' else 'release')
VERSION_NAME = VERSION_SCHEME['.'.join(VERSION.split('.')[:2])]

# ANCHOR: Vendor

# If you fork Sea, you should change this to you or your organization's name.
VENDOR = 'official'
IS_OFFICIAL = VENDOR == 'official'

LICENSE = 'MIT'

# The implementation language of the compiler
IMPLEMENTATION = 'python'

# ANCHOR: Directories

GLOBAL_DIR = pathlib.Path.home() / '.sea'
GLOBAL_LIB_DIR = GLOBAL_DIR / 'lib'

LOCAL_DIR = pathlib.Path('.sea')
LOCAL_BUILD_DIR = LOCAL_DIR / 'build'

# ANCHOR: Compilation

COMPILER_DEBUG = 'tcc'
FLAGS_DEBUG = '-g3'

COMPILER_PROD = 'gcc' # can also be `clang`
FLAGS_PROD = '-O3'
