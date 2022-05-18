#!/usr/bin/env python3
from datetime import datetime
import pathlib
import sys

from yapapi import (
    Golem,
    Task,
)
from yapapi.payload import vm

examples_dir = pathlib.Path(__file__).resolve().parent.parent
sys.path.append(str(examples_dir))

from a1_requestor.yapapi_example_utils import (
    build_parser,
    run_golem_example,
    print_env_info,
)
from a1_requestor.worker import worker, IMAGE_HASH, prepare_task_data, TaskTimeout, IncorrectResult
from a1_requestor.strategy import AlphaRequestorStrategy


async def main(golem, *, num_providers, task_size):
    payload = await vm.repo(image_hash=IMAGE_HASH)

    async with golem:
        print_env_info(golem)

        task_data = prepare_task_data(task_size)
        tasks = [Task(data=task_data) for _ in range(num_providers)]

        num_tasks = 0
        async for _ in golem.execute_tasks(worker, tasks, payload, max_workers=num_providers):
            num_tasks += 1
            print(f"{num_tasks} out of {num_providers} planned providers tested")


if __name__ == "__main__":
    parser = build_parser("Run an alpha1 requestor")
    now = datetime.now().strftime("%Y-%m-%d_%H.%M.%S")
    parser.set_defaults(log_file=f"a1_requestor-{now}.log")
    args = parser.parse_args()

    strategy = AlphaRequestorStrategy(min_offers=6, repu_factor=0)
    golem = Golem(
        budget=10,  # TODO: do we need to parametrize this?
        strategy=strategy,
        subnet_tag=args.subnet_tag,
        payment_driver=args.payment_driver,
        payment_network=args.payment_network,
    )
    golem.add_event_consumer(
        strategy.event_consumer,
        ["ProposalReceived", TaskTimeout, IncorrectResult, "TaskAccepted"]
    )

    run_golem_example(
        main(golem, num_providers=3, task_size=7),
        log_file=args.log_file,
    )

    print(f"PAYABLE: {len(strategy._payable_activities)}, FAILED: {len(strategy._failed_activities)}")
