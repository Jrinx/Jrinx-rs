#!/usr/bin/env python3

from __future__ import annotations

import argparse
from collections import OrderedDict
import itertools
import subprocess
from typing import Callable, Sequence

import pathos.multiprocessing as mp

from util import *

TESTS_DIR = dir_ancestor_find(
    pathlib.Path(__file__),
    'scripts',
).parent / 'tests'


def run_test(file: pathlib.Path,
             include_dirs: list[pathlib.Path],
             board: str,
             /, *,
             verbose: bool = False,
             pre_run: Callable[[], None],
             on_success: Callable[[], None],
             on_failure: Callable[[], None]) -> int:
    pre_run()
    cmd = [
        dir_ancestor_find(pathlib.Path(__file__), 'scripts') / 'judge',
        '-f', str(file),
    ]
    if verbose:
        cmd.append('-v')
    for inc_dir in include_dirs:
        cmd.extend(('-I', str(inc_dir)))
    try:
        env = dict(os.environ, BOARD=board)
        if verbose:
            subprocess.check_call(cmd, env=env)
        else:
            subprocess.check_call(cmd, env=env, stderr=subprocess.DEVNULL)
    except subprocess.CalledProcessError as e:
        on_failure()
        return e.returncode
    else:
        on_success()
        return 0


def run_testset(testset: tuple[pathlib.Path],
                include_dirs: list[pathlib.Path],
                board: tuple[str],
                /, *,
                parallel: bool = False,
                verbose: bool = False,
                ) -> Sequence[int]:
    info(
        f'Run testset on {os.environ["ARCH"]} in {os.environ["BUILD_MODE"]} mode'
    )

    def run(arg) -> int:
        file, board = arg[0], arg[1]
        slug = str(file.relative_to(TESTS_DIR))
        return run_test(
            file,
            include_dirs,
            board,
            verbose=verbose,
            pre_run=lambda: info(f'On {board:<14} Run    {slug}'),
            on_success=lambda: info(f'On {board:<14} Passed {slug}'),
            on_failure=lambda: fatal(f'On {board:<14} Failed {slug}'),
        )

    comb = tuple(itertools.product(testset, board))

    result = mp.ThreadingPool(
        mp.cpu_count() if parallel else 1
    ).map(run, comb)

    if any(result):
        failed_testset = tuple((
            str(test.relative_to(TESTS_DIR)),
            board
        ) for ((test, board), ret) in zip(comb, result) if ret != 0)
        fatal(f'Failed in {failed_testset}')

    return result


def run_testset_rich(testset: tuple[pathlib.Path],
                     include_dirs: list[pathlib.Path],
                     board: tuple[str],
                     /, *,
                     parallel: bool = False,
                     verbose: bool = False,
                     ) -> Sequence[int]:
    from rich.align import Align
    from rich.console import Console
    from rich.live import Live
    from rich.table import Table
    from rich.text import Text

    comb = tuple(itertools.product(testset, board))

    test_status = OrderedDict((
        (str(test.relative_to(TESTS_DIR)), board),
        'waiting',
    ) for (test, board) in comb)

    def gen_table():
        table = Table('Test', 'Board',
                      'Status (waiting|running|passed|failed)')
        table.title = Text(
            f'Run testset on {os.environ["ARCH"]} in {os.environ["BUILD_MODE"]} mode',
            style='bold',
        )
        for (test, board), status in test_status.items():
            table.add_row(
                test,
                board,
                Align.center(Text(
                    status,
                    style='white' if status == 'waiting'
                    else 'yellow' if status == 'running'
                    else 'green' if status == 'passed' else 'red',
                )),
            )
        return Align.center(table)

    console = Console()

    with Live(
        gen_table(),
        console=console,
        screen=False,
        refresh_per_second=1,
    ) as live:
        def run(arg):
            file, board = arg[0], arg[1]
            slug = (str(file.relative_to(TESTS_DIR)), board)

            def pre_run():
                test_status[slug] = 'running'
                live.update(gen_table())

            def on_success():
                test_status[slug] = 'passed'
                live.update(gen_table())

            def on_failure():
                test_status[slug] = 'failed'
                live.update(gen_table())

            return run_test(
                file,
                include_dirs,
                board,
                verbose=verbose,
                pre_run=pre_run,
                on_success=on_success,
                on_failure=on_failure,
            )

        result = mp.ThreadingPool(
            mp.cpu_count() if parallel else 1
        ).map(run, comb)

    console.print('')  # newline

    return result


def main():
    args = argparse.ArgumentParser()
    args.add_argument('-f', '--file', type=file_path)
    args.add_argument('-I', '--include-dir',
                      type=dir_path,
                      action='append',
                      default=[],
                      )
    args.add_argument('-n', '--no-build', action='store_true')
    args.add_argument('-p', '--parallel', action='store_true')
    args.add_argument('-v', '--verbose', action='store_true')
    args.add_argument('-r', '--rich', action='store_true')
    args = args.parse_args()

    include_dirs: list[pathlib.Path] = [
        TESTS_DIR / 'include',
        *args.include_dir,
    ]

    setup_envs()

    if args.file:
        testset = tuple((args.file.resolve(),))
    else:
        testset = tuple(itertools.chain(
            (TESTS_DIR / 'kern').rglob('*.yml'),
            (TESTS_DIR / 'kern').rglob('*.yaml'),
        ))

    if not args.no_build:
        subprocess.check_call(('cargo', 'make'))

    runner = run_testset_rich if args.rich else run_testset

    exit_code = any(runner(testset, include_dirs, read_board_list(
        os.environ['ARCH']), parallel=args.parallel, verbose=args.verbose))

    exit(exit_code)


if __name__ == '__main__':
    main()
