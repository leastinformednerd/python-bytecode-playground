from argparse import ArgumentParser
from unify import create_pyc_from_program
from pathlib import PurePath

argparser = ArgumentParser()
argparser.add_argument("-o", "--output")
argparser.add_argument("-d", "--dis", action="store_true")
argparser.add_argument("filename")

args = argparser.parse_args()

create_pyc_from_program(
    args.filename,
    args.output if args.output is not None else PurePath(args.filename).with_suffix(".pyc").as_posix(),
    args.dis
)
