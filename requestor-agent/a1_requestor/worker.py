from datetime import timedelta
import math
from random import randint, random
from typing import Dict, List

from primefac import isprime

from yapapi import WorkContext
from yapapi.rest.activity import BatchTimeoutError
from yapapi.events import ActivityEvent

IMAGE_DOWNLOAD_TIMEOUT = timedelta(minutes=3)

#   Python 3.8-alpine + primefac
IMAGE_HASH = "1ff5f0fe16b6a1f3fc7a01eba370d10845b21f3745e25eeba6dc5340"

#   Task that is used if task_data == 0
DEBUG_TASK_DATA = {98320228208849205109158097780301701: [403168042031, 421910160769, 578011918859]}


class TaskTimeout(ActivityEvent):
    pass


class IncorrectResult(ActivityEvent):
    pass


def get_random_primes(cnt, min_size, max_size):
    """Return CNT prime numbers in range (MIN_SIZE, MAX_SIZE)"""
    result = []
    while len(result) < cnt:
        x = randint(2, max_size)
        if isprime(x):
            result.append(x)
    return result


def prepare_task_data(task_size: int) -> Dict[int, List[int]]:
    """Create a random map {integer -> list_of_prime_factors} with TASK_SIZE elements"""
    if task_size < 1:
        return DEBUG_TASK_DATA

    src_data = {}
    for i in range(task_size):
        #   With this params it takes ~~ 10-150s to factorize a single number on the devnet-beta,
        primes = get_random_primes(3, 10**15, 10**18)
        num = math.prod(primes)
        if num not in src_data:
            src_data[num] = sorted(primes)

    return src_data


def parse_stdout(provider_stdout: str) -> Dict[int, List[int]]:
    """Create a map {integer -> list_of_prime_factors} based on the PROVIDER_STDOUT"""
    data = {}
    for line in provider_stdout.splitlines():
        number_str, primes_str = line.split(':')
        number = int(number_str.strip())
        primes = [int(x) for x in primes_str.strip().split()]
        data[number] = sorted(primes)
    return data


async def worker(ctx: WorkContext, tasks):
    #   Separate pre-task so we have a seprate timeout for download
    script = ctx.new_script(timeout=IMAGE_DOWNLOAD_TIMEOUT)
    script.run("/bin/sh", "-c", "echo foo")
    try:
        yield script
    except BatchTimeoutError:
        ctx.emit(TaskTimeout)
        raise

    try:
        task = await tasks.__anext__()
    except StopAsyncIteration:
        #   I'm not sure if this is possible?
        return

    src_data, random_fail_factor, task_timeout_factor = task.data

    command_args_str = " ".join(str(x) for x in sorted(src_data))
    command = ["/bin/sh", "-c", f"python3 -m primefac {command_args_str}"]
    timeout = timedelta(seconds=len(src_data) * task_timeout_factor)

    script = ctx.new_script(timeout=timeout)
    result = script.run(*command)

    try:
        yield script
    except BatchTimeoutError:
        ctx.emit(TaskTimeout)
        raise

    try:
        received_data = parse_stdout((await result).stdout)
        assert received_data == src_data
    except Exception:
        ctx.emit(IncorrectResult)
        raise

    if random_fail_factor > 100 * random():
        ctx.emit(IncorrectResult)
        raise Exception("Task randomly pseud-failed")

    task.accept_result()

    #   This stops the tasks API engine from reusing this agreement for other tasks
    await tasks.aclose()
