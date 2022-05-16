from yapapi import WorkContext
from yapapi.rest.activity import BatchTimeoutError
from datetime import timedelta


IMAGE_DOWNLOAD_TIMEOUT = timedelta(minutes=3)

#   Now blender, but this will change
IMAGE_HASH = "9a3b5d67b0b27746283cb5f287c13eab1beaa12d92a9f536b747c7ae"


def prepare_task_data(task_size: int):
    numbers = " ".join([str(x) for x in range(task_size)])
    command = ["/bin/bash", "-c", f"echo -n {numbers}"]
    expected_output = numbers
    timeout = timedelta(minutes=1)

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

    print("RUNNIG A TASK")
    yield script

    received_output = (await result).stdout
    if received_output != expected_output:
        raise Exception("BAD PROVIDER RETURNED A BAD RESULT!")

    task.accept_result()

    #   This stops the tasks API engine from reusing this agreement for other tasks
    await tasks.aclose()
