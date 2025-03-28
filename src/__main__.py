import os
from typing import Optional
import click
from . import visitor

def init_dirs():
	if not os.path.exists('.sea'):
		os.mkdir('.sea')
		os.mkdir('.sea/build/')

@click.command()
@click.option('--output', default='main', help='Output file', type=click.Path(file_okay=True))
@click.option('--prod', is_flag=True, help='Toggle production optimizations (-O3 on GCC/Clang)')
@click.option('--cc', default=None, type=Optional[str], help='The compiler to use by default')
@click.option('--ccflags', default='', help='Options to pass to the C compiler')
@click.option('--nobuild', is_flag=True, help='Makes Sea only skip building and only transpile')
@click.option('--libpaths', default='.:~/.sea/lib/', help='Paths to each directory that should be searched for libraries, the first path is searched first')
@click.option('--run', is_flag=True, help='Run the file immediately after compiling it')
@click.option('--args', type=str, help='Arguments to pass to the program')
@click.argument('input', type=click.Path(exists=True, file_okay=True))
def cli(output, prod, cc, ccflags, nobuild, libpaths, run, args, input):
	'''Build the given input file'''
	init_dirs()

	visitor.visit(input, output_path = '.sea/build/output.c')

	if nobuild:
		return

	cc = 'tcc'
	if prod and cc is None:
		cc = 'gcc'

	command = f'{cc} {ccflags} -o {output} .sea/build/output.c'
	print(': ' + command)
	result = os.system(command)

	if run and result == 0:
		command = f'./{output} {args}'
		print(': ' + command)
		os.system(command)

if __name__ == "__main__":
	cli()
