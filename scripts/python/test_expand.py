#!/usr/bin/env python3

import argparse
import itertools
import os
import sys

import expandvars
import yaml

try:
    from yaml import CLoader as Loader
except ImportError:
    from yaml import Loader

from util import *

TESTS_DIR = dir_ancestor_find(pathlib.Path(
    __file__), 'scripts').parent / 'tests'


def expand(conf: dict, /, *, extra_envs: dict[str, str], inc_dirs: list[pathlib.Path]) -> dict:
    for k, v in conf.items():
        if k == 'include':
            if not isinstance(v, str):
                raise TypeError(f'include "{v}" is not a string')
            inc_files = (file
                         for inc_dir in inc_dirs
                         for file in itertools.chain(
                             inc_dir.rglob('*.yml'),
                             inc_dir.rglob('*.yaml'),
                         ))
            inc_file = next((f for f in inc_files if f.name in (
                v, f'{v}.yml', f'{v}.yaml')), None)
            if inc_file is None:
                raise FileNotFoundError(f'include "{v}" not found')
            with open(inc_file, 'r', encoding='utf-8') as f:
                inc_conf = yaml.load(f, Loader=Loader)
            conf = conf.copy()
            del conf[k]
            return expand(conf | inc_conf, extra_envs=extra_envs, inc_dirs=inc_dirs)

    for k, v in conf.items():
        def do_expand(v):
            if isinstance(v, dict):
                return expand(v, extra_envs=extra_envs, inc_dirs=inc_dirs)
            elif isinstance(v, str):
                return expandvars.expand(v,
                                         nounset=True,
                                         environ={
                                             **os.environ,
                                             **extra_envs,
                                         },
                                         )
            elif isinstance(v, list):
                return [do_expand(val) for val in v]
            else:
                raise TypeError(f'"{v}" is not a string, dict, or list')
        conf[k] = do_expand(v)

    return conf


def main():
    args = argparse.ArgumentParser()
    args.add_argument('-f', '--file',
                      type=file_path,
                      required=True,
                      )
    args.add_argument('-I', '--include-dir',
                      type=dir_path,
                      action='append',
                      default=[],
                      )
    args = args.parse_args()

    include_dirs: list[pathlib.Path] = [
        TESTS_DIR / 'include',
        *args.include_dir,
    ]

    with open(args.file, 'r', encoding='utf-8') as f:
        conf = yaml.load(f, Loader=Loader)
    conf = expand(conf,
                  extra_envs={
                      'TEST_NAME': args.file.stem,
                  },
                  inc_dirs=[dir for dir in include_dirs if dir.is_dir()],
                  )

    yaml.dump(conf, sys.stdout)


if __name__ == '__main__':
    main()
