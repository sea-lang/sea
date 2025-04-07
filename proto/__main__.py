import os
import click
from .reef import ReefSchemaBuilder
from .backend_c import Backend_C
from .compiler import Compiler
from . import config
from . import visitor


CONFIG_SCHEMA = (ReefSchemaBuilder()
	.field_string('sea.cc.flags')
	.field_bool('sea.nostd', default = 'false')
	.build())


def init_dirs():
	if not os.path.exists(config.LOCAL_DIR):
		os.mkdir(config.LOCAL_DIR)
		os.mkdir(config.LOCAL_BUILD_DIR)

def run_cmd(cmd: str) -> int:
	print(f': {cmd}')
	return os.system(cmd)


@click.command()
@click.option('-o', '--output', default='main', help='Output file', type=click.Path(file_okay=True))
@click.option('-p', '--prod', is_flag=True, help='Toggle production optimizations (-O3 on GCC/Clang)')
@click.option('-c', '--cc', default=None, type=str, help='The compiler to use by default')
@click.option('-f', '--ccflags', default=None, help='Options to pass to the C compiler')
@click.option('-n', '--nobuild', is_flag=True, help='Makes Sea only skip building and only transpile')
@click.option('-l', '--libpaths', default=None, type=str, help='Paths to each directory that should be searched for libraries, the first path is searched first')
@click.option('-r', '--run', is_flag=True, help='Run the file immediately after compiling it')
@click.option('-a', '--args', default='', type=str, help='Arguments to pass to the program')
@click.option('-s', '--std', default=None, type=str, help='Path to the standard library')
@click.option('-S', '--nostd', is_flag=True, help='Disable implicit `use std`')
@click.argument('input', type=click.Path(exists=True, file_okay=True))
def cli(output, prod, cc, ccflags, nobuild, libpaths, run, args, std, nostd, input):
	'''Build the given input file'''

	init_dirs()

	if libpaths is None:
		libpaths = f'{os.path.dirname(input)}:{std or config.GLOBAL_LIB_DIR}'
	if cc is None:
		cc = config.COMPILER_PROD if prod else config.COMPILER_DEBUG
	if ccflags is None:
		ccflags = config.FLAGS_PROD if prod else config.FLAGS_DEBUG

	build_reef = os.path.dirname(input) + '/build.reef'
	if os.path.exists(build_reef):
		conf = CONFIG_SCHEMA.parse_file(build_reef)
		if not conf.success:
			print('error reading build.reef: ' + conf.message)
			exit(1)

		ccflags += conf.fields['sea.cc.flags']

		if conf.fields['sea.nostd'] == 'true':
			nostd = True

	output_c = str(config.LOCAL_BUILD_DIR / 'output.c')
	with Backend_C(Compiler(), output_c) as backend:
		backend.libpaths = libpaths.split(':')
		visitor.visit(input, backend = backend, nostd = nostd)

	if not nobuild:
		result = run_cmd(f'{cc} {ccflags} -o {output} {output_c}')
		if run and result == 0:
			run_cmd(f'./{output} {args}')


if __name__ == '__main__':
	cli()
