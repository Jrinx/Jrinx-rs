#!/usr/bin/env python3

from __future__ import annotations

from abc import abstractmethod
import argparse
import signal
import subprocess
import re

import yaml

try:
    from yaml import CLoader as Loader
except ImportError:
    from yaml import Loader

import test_expand
from util import *


TESTS_DIR = dir_ancestor_find(
    pathlib.Path(__file__),
    'scripts',
).parent / 'tests'


class Pattern:
    def __init__(self, pattern: str):
        self.pattern = pattern

    @abstractmethod
    def __repr__(self) -> str:
        ...

    @abstractmethod
    def apply(self, s: str) -> tuple[bool, tuple[str]]:
        ...

    @classmethod
    def of(cls, pattern) -> Pattern | None:
        if pattern is None:
            return None
        match pattern:
            case str():
                return StringPattern(pattern)
            case {'type': typ, **__}:
                return eval(f'{typ.title()}Pattern')(pattern)
            case _:
                raise ValueError(
                    f'Invalid pattern "{pattern}" with type {type(pattern)}'
                )


class StringPattern(Pattern):
    def __init__(self, pattern: str):
        super().__init__(pattern)
        self.__regex: re.Pattern = re.compile(pattern)

    def __repr__(self) -> str:
        return f'StringPattern({self.__regex})'

    def apply(self, s: str) -> tuple[bool, tuple[str]]:
        mat = self.__regex.search(s)
        if mat:
            return True, (mat[0],)
        else:
            return False, ()


class OrderedPattern(Pattern):
    def __init__(self, pattern: str):
        super().__init__(pattern)
        self.__pats = tuple(Pattern.of(p) for p in pattern['vals'])
        self.__next = 0

    def __repr__(self) -> str:
        return f'OrderedPattern({self.__pats})'

    def apply(self, s: str) -> tuple[bool, tuple[str]]:
        retire, picked_strs = self.__pats[self.__next].apply(s)
        if picked_strs:
            if retire:
                self.__next += 1
            return self.__next >= len(self.__pats), picked_strs
        return False, ()


class RepeatPattern(Pattern):
    def __init__(self, pattern: str):
        super().__init__(pattern)
        self.__origin_pat_obj = pattern['pat']
        self.__pat = Pattern.of(self.__origin_pat_obj)
        self.__count = pattern['count']
        self.__rem = self.__count

    def __repr__(self) -> str:
        return f'RepeatPattern({self.__pat}, {self.__count})'

    def apply(self, s: str) -> tuple[bool, tuple[str]]:
        retire, picked_strs = self.__pat.apply(s)
        if picked_strs:
            if retire:
                self.__rem -= 1
                self.__pat = Pattern.of(self.__origin_pat_obj)
        return self.__rem == 0, picked_strs


class UnorderedPattern(Pattern):
    def __init__(self, pattern: str):
        super().__init__(pattern)
        self.__pats = frozenset(Pattern.of(p) for p in pattern['vals'])
        self.__waiting_pats = set(self.__pats)

    def __repr__(self) -> str:
        return f'UnorderedPattern({self.__pats})'

    def apply(self, s: str) -> tuple[bool, tuple[str]]:
        remove_pats = set()
        all_picked_strs = []
        for pat in self.__waiting_pats:
            retire, picked_strs = pat.apply(s)
            if picked_strs:
                all_picked_strs += picked_strs
                if retire:
                    remove_pats.add(pat)
        self.__waiting_pats -= remove_pats
        return len(self.__waiting_pats) == 0, tuple(all_picked_strs)


class Test:
    TIMEOUT = 180

    def load_from_file(file: pathlib.Path, include_dirs: list[pathlib.Path]) -> Test:
        with file.open('r', encoding='utf-8') as f:
            conf = yaml.load(f, Loader=Loader)
        if (parts := list(file.relative_to(TESTS_DIR).parts))[0] == 'kern':
            parts[-1] = pathlib.Path(parts[-1]).stem
            test_name = 'jrinx::test::' + '::'.join(parts[1:])
            conf = test_expand.expand(
                conf,
                extra_envs={
                    'TEST_NAME': test_name,
                },
                inc_dirs=[dir for dir in include_dirs if dir.is_dir()],
            )
            return Test(conf, f'-t {test_name}')
        else:
            raise NotImplementedError()

    def __init__(self, conf: dict, bootargs: str | None):
        self.conf = conf
        self.bootargs = bootargs

    def __call__(self, /, *, verbose: bool = False):
        output = []
        expected_pattern = Pattern.of(self.conf.get('expected'))
        unexpected_pattern = Pattern.of(self.conf.get('unexpected'))

        if not expected_pattern:
            raise ValueError('No expected pattern specified')

        try:
            env = os.environ.copy()
            if self.bootargs:
                if (args := env.get('BOOTARGS')) is not None:
                    env['BOOTARGS'] = args + ' ' + self.bootargs
                else:
                    env['BOOTARGS'] = self.bootargs
            proc = subprocess.Popen(('cargo', 'qemu'),
                                    env=env,
                                    stdin=subprocess.PIPE,
                                    stdout=subprocess.PIPE,
                                    stderr=subprocess.STDOUT,
                                    start_new_session=True,
                                    )

            def on_timeout(*_):
                if proc:
                    eliminate_child(proc,
                                    timeout=Test.TIMEOUT,
                                    verbose=verbose,
                                    )
                raise RuntimeError('Timeout')

            signal.signal(signal.SIGALRM, on_timeout)
            signal.alarm(Test.TIMEOUT)

            for out in proc.stdout:
                line = out if isinstance(out, str) else out.decode('utf-8')
                output.append(line)
                if verbose:
                    sys.stdout.write(line)
                if unexpected_pattern:
                    retire, picked_strs = unexpected_pattern.apply(line)
                    if picked_strs and retire:
                        raise RuntimeError(
                            f'Unexpected pattern "{unexpected_pattern}" found in line "{line}"'
                        )
                retire, picked_strs = expected_pattern.apply(line)
                if picked_strs:
                    if verbose:
                        info(f'Picked strings {picked_strs}')
                    if retire:
                        if verbose:
                            info('Expected pattern defined found')
                        return
            raise RuntimeError('Expected pattern not found')
        finally:
            signal.alarm(0)
            eliminate_child(proc, timeout=Test.TIMEOUT, verbose=verbose)


def judge(file: pathlib.Path,
          include_dirs: list[pathlib.Path],
          /, *,
          build: bool = False,
          verbose: bool = False) -> int:
    if build:
        subprocess.check_call(('make', 'build'),
                              env=dict(
                                  os.environ,
                                  VERBOSE='true' if verbose else 'false',
        ))
    test = Test.load_from_file(file, include_dirs)
    try:
        test(verbose=verbose)
    except RuntimeError:
        return 1
    return 0


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
    args.add_argument('-B', '--build', action='store_true')
    args.add_argument('-v', '--verbose', action='store_true')
    args = args.parse_args()

    file = args.file.resolve()
    include_dirs: list[pathlib.Path] = [
        TESTS_DIR / 'include',
        *args.include_dir,
    ]

    setup_envs()

    exit(judge(file, include_dirs, build=args.build, verbose=args.verbose))


if __name__ == '__main__':
    main()
