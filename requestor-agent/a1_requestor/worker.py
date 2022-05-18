from datetime import timedelta
import math
from random import randint
from typing import Dict, List, Tuple

from primefac import isprime

from yapapi import WorkContext
from yapapi.rest.activity import BatchTimeoutError


IMAGE_DOWNLOAD_TIMEOUT = timedelta(minutes=3)

#   Python 3.8-alpine + primefac
IMAGE_HASH = "1ff5f0fe16b6a1f3fc7a01eba370d10845b21f3745e25eeba6dc5340"


def get_random_primes(cnt, max_size):
    result = []
    while len(result) < cnt:
        x = randint(2, max_size)
        if isprime(x):
            result.append(x)
    return result


def prepare_task_data(task_size: int) -> Tuple[List[str], Dict[int, List[int]], timedelta]:
    #   Create (int, prime_factors) map
    src_data = {}
    for i in range(task_size):
        #   With this params it takes ~~ 1s to factorize the number on the devnet-beta,
        #   so task_size ~~ time_in_seconds.
        primes = get_random_primes(8, 10**10)
        num = math.prod(primes)
        if num not in src_data:
            src_data[num] = sorted(primes)

    command_args_str = " ".join(str(x) for x in sorted(src_data))
    command = ["/bin/sh", "-c", f"python3 -m primefac {command_args_str}"]

    timeout = timedelta(task_size * 5)

    return (command, src_data, timeout)


def parse_stdout(provider_stdout: str) -> Dict[int, List[int]]:
    data = {}
    for line in provider_stdout.splitlines():
        number_str, primes_str = line.split(':')
        number = int(number_str.strip())
        primes = [int(x) for x in primes_str.strip().split()]
        data[number] = sorted(primes)
    return data


async def worker(ctx: WorkContext, tasks):
    script = ctx.new_script(timeout=IMAGE_DOWNLOAD_TIMEOUT)
    script.run("/bin/sh", "-c", "echo foo")
    try:
        yield script
    except BatchTimeoutError:
        #   TODO: save the information that provider failed
        raise

    try:
        task = await tasks.__anext__()
    except StopAsyncIteration:
        #   I'm not sure if this is possible?
        return

    command, src_data, timeout = task.data

    script = ctx.new_script(timeout=timeout)
    result = script.run(*command)

    yield script

    received_data = parse_stdout((await result).stdout)
    if received_data != src_data:
        raise Exception("BAD PROVIDER RETURNED A BAD RESULT!")

    task.accept_result()

    #   This stops the tasks API engine from reusing this agreement for other tasks
    await tasks.aclose()
