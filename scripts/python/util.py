import argparse
import datetime
import os
import pathlib
import psutil
import random
import subprocess
import sys

if sys.stdout.isatty() or 'GITLAB_CI' in os.environ:
    def make_printer(prefix, _, *, file=sys.stdout):
        def wrapper(s, file=file):
            print(f'{prefix}>>> {s}\033[0m', file=file)

        return wrapper
else:
    def make_printer(_, level, *, file=sys.stdout):
        def wrapper(s, file=file):
            plain = f'[{level}] {s}'
            print(plain, file=file)

        return wrapper


info = make_printer('\033[32m', 'INFO')
warn = make_printer('\033[33m', 'WARN', file=sys.stderr)
fatal = make_printer('\033[31m', 'FATAL', file=sys.stderr)


def file_path(path: str) -> pathlib.Path:
    if os.path.isfile(path):
        return pathlib.Path(path)
    else:
        raise argparse.ArgumentTypeError(f'"{path}" is not a file')


def dir_path(path: str) -> pathlib.Path:
    if os.path.isdir(path):
        return pathlib.Path(path)
    else:
        raise argparse.ArgumentTypeError(f'"{path}" is not a directory')


def dir_ancestor_find(path: pathlib.Path, name: str) -> pathlib.Path:
    while path != path.root:
        if path.is_dir() and path.name == name:
            return path
        path = path.parent


def eliminate_child(ch: subprocess.Popen[bytes], /, *, timeout: int, verbose: bool = False):
    try:
        proc = psutil.Process(ch.pid)
    except psutil.NoSuchProcess:
        return

    if ch.stdin:
        try:
            ch.stdin.close()
        except RuntimeError:
            pass

    if ch.stdout:
        try:
            if verbose:
                os.set_blocking(ch.stdout.fileno(), False)
                b = ch.stdout.read()
                if b:
                    sys.stdout.buffer.write(b)
            ch.stdout.close()

        except RuntimeError:
            pass

    chs = [proc]
    chs.extend(proc.children(recursive=True))

    for p in chs:
        try:
            p.terminate()
        except psutil.NoSuchProcess:
            pass

    _, alive = psutil.wait_procs(chs, timeout=timeout)
    for p in alive:
        try:
            p.kill()
        except psutil.NoSuchProcess:
            pass

    ch.wait()


def setup_envs() -> None:
    envs = {
        'ARCH': 'riscv64',
        'BOARD': 'virt',
        'BUILD_MODE': 'release',
        'BUILD_TIME': datetime.datetime.now().strftime(r'%Y-%m-%d %H:%M:%S'),
        'RAND_SEED': str(random.randint(0, 32767)),
        'SMP': '5',
        'MEMORY': '1G',
    }

    for key, value in envs.items():
        os.environ[key] = os.environ.get(key, value)


def read_board_list(arch: str) -> list[str]:
    board_list_dir = dir_ancestor_find(
        pathlib.Path(__file__),
        'scripts',
    ).parent / 'kern' / 'tgt'

    while not (board_list_file := board_list_dir / f'{arch}.board-list').exists():
        arch = arch[:-1]

    with board_list_file.open() as f:
        return f.read().splitlines()
