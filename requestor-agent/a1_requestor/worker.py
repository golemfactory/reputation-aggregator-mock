from datetime import timedelta
import math
from random import randint

from primefac import isprime

from yapapi import WorkContext
from yapapi.rest.activity import BatchTimeoutError


IMAGE_DOWNLOAD_TIMEOUT = timedelta(minutes=3)

#   Now blender, but this will change
IMAGE_HASH = "9a3b5d67b0b27746283cb5f287c13eab1beaa12d92a9f536b747c7ae"


def get_random_primes(cnt, max_size):
    result = []
    while len(result) < cnt:
        x = randint(2, max_size)
        if isprime(x):
            result.append(x)
    return result


def prepare_task_data(task_size: int):
    #   Create (int, prime_factors) map
    raw_data = {}
    for i in range(task_size):
        primes = get_random_primes(3, 10**6)
        num = math.prod(primes)
        if num not in raw_data:
            raw_data[num] = primes

    #   Convert the map to a command and expected output
    command_args = []
    output_lines = []
    for num in sorted(raw_data):
        primes_str = " ".join(str(p) for p in sorted(raw_data[num]))
        command_args.append(num)
        output_lines.append(f"{num}: {primes_str}")

    command_args_str = " ".join(str(x) for x in command_args)
    command = ["/bin/bash", "-c", f"factor {command_args_str}"]

    expected_output = "\n".join(output_lines) + "\n"
    timeout = timedelta(seconds=10 + task_size)

    return (command, expected_output, timeout)


async def worker(ctx: WorkContext, tasks):
    script = ctx.new_script(timeout=IMAGE_DOWNLOAD_TIMEOUT)
    script.run("/bin/bash", "-c", "echo foo")
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

    command, expected_output, timeout = task.data

    script = ctx.new_script(timeout=timeout)
    result = script.run(*command)

    yield script

    received_output = (await result).stdout
    if received_output != expected_output:
        raise Exception("BAD PROVIDER RETURNED A BAD RESULT!")

    task.accept_result()

    #   This stops the tasks API engine from reusing this agreement for other tasks
    await tasks.aclose()
